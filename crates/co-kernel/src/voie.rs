//! Voies: ranks, voie and ability identifiers, and the checks on the ranks
//! acquired in a voie.

use std::{collections::BTreeSet, ops::RangeInclusive};

use crate::{Mechanic, RuleId, Severity, Subject, Violation};

/// The rank of an ability within a voie.
///
/// No game bounds: ranks 1 to 5 (CO.VOIE.STRUCTURE) or 4 to 8 for prestige
/// voies are rules checked by validation, not by this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rank(u8);

impl Rank {
    /// Creates a rank.
    #[must_use]
    pub const fn new(rank: u8) -> Self {
        Self(rank)
    }

    /// Returns the rank number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl From<u8> for Rank {
    fn from(rank: u8) -> Self {
        Self(rank)
    }
}

/// Identifier of a voie.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoieId(String);

impl VoieId {
    /// Creates an identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identifier of an ability (capacité).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbilityId(String);

impl AbilityId {
    /// Creates an identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for AbilityId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for AbilityId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// CO.VOIE.STRUCTURE: the ranks of a voie run from 1 to 5.
pub const VOIE_STRUCTURE: RuleId = RuleId::from_static("CO.VOIE.STRUCTURE");

/// CO.VOIE.ORDER: the lower ranks of a voie are acquired first, with no gap.
pub const VOIE_ORDER: RuleId = RuleId::from_static("CO.VOIE.ORDER");

/// Lowest and highest rank of a voie (CO.VOIE.STRUCTURE).
const RANKS: RangeInclusive<u8> = 1..=5;

/// Checks the `ranks` acquired in `voie`, for `mechanic`: a `standard`
/// deviation of [`VOIE_STRUCTURE`] for each rank outside 1 to 5, and one of
/// [`VOIE_ORDER`] when ranks are missing below the highest, with the first
/// missing rank and their count.
#[must_use]
pub fn voie_rank_violations(
    voie: &VoieId,
    ranks: &BTreeSet<Rank>,
    mechanic: Mechanic,
) -> Vec<Violation> {
    let subject = Subject::Voie(voie.clone());
    let mut violations: Vec<Violation> = ranks
        .iter()
        .filter(|rank| !RANKS.contains(&rank.get()))
        .map(|rank| {
            Violation::deviation(
                VOIE_STRUCTURE,
                mechanic,
                subject.clone(),
                Severity::Standard,
            )
            .with_parameter("rank", i32::from(rank.get()))
        })
        .collect();
    let highest = ranks.last().map_or(0, |rank| rank.get());
    let mut missing = (1..highest)
        .filter(|r| !ranks.contains(&Rank::new(*r)))
        .peekable();
    if let Some(&first) = missing.peek() {
        let count = missing.fold(0_i32, |count, _| count + 1);
        violations.push(
            Violation::deviation(VOIE_ORDER, mechanic, subject.clone(), Severity::Standard)
                .with_parameter("first_missing", i32::from(first))
                .with_parameter("missing", count),
        );
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Category, ParameterValue};

    const MECHANIC: Mechanic = Mechanic::new("test-mechanic");

    fn subject() -> Subject {
        Subject::Voie(VoieId::new("test-voie"))
    }

    fn check(ranks: &[u8]) -> Vec<Violation> {
        let ranks: BTreeSet<Rank> = ranks.iter().copied().map(Rank::new).collect();
        voie_rank_violations(&VoieId::new("test-voie"), &ranks, MECHANIC)
    }

    fn parameters(violation: &Violation) -> Vec<(&str, &ParameterValue)> {
        violation
            .parameters()
            .iter()
            .map(|p| (p.name(), p.value()))
            .collect()
    }

    #[test]
    fn ranks_acquired_in_order_are_clean() {
        assert_eq!(check(&[]), []);
        assert_eq!(check(&[1]), []);
        assert_eq!(check(&[1, 2, 3, 4, 5]), []);
    }

    #[test]
    fn each_rank_outside_the_structure_deviates() {
        let violations = check(&[0, 1, 2, 3, 4, 5, 6]);
        let ranks: Vec<_> = violations
            .iter()
            .map(|v| {
                assert_eq!(v.rule(), &VOIE_STRUCTURE);
                parameters(v)
            })
            .collect();
        assert_eq!(
            ranks,
            [
                [("rank", &ParameterValue::Integer(0))],
                [("rank", &ParameterValue::Integer(6))]
            ]
        );
    }

    #[test]
    fn gaps_make_one_order_deviation() {
        let violations = check(&[2, 5]);
        assert_eq!(violations.len(), 1);
        let order = &violations[0];
        assert_eq!(order.rule(), &VOIE_ORDER);
        assert_eq!(order.category(), Category::Deviation(Severity::Standard));
        assert_eq!(order.mechanic(), MECHANIC);
        assert_eq!(order.subject(), &subject());
        assert_eq!(
            parameters(order),
            [
                ("first_missing", &ParameterValue::Integer(1)),
                ("missing", &ParameterValue::Integer(3))
            ]
        );
    }

    #[test]
    fn a_rank_far_out_deviates_twice() {
        let rules: Vec<_> = check(&[200])
            .iter()
            .map(|v| v.rule().as_str().to_owned())
            .collect();
        assert_eq!(rules, ["CO.VOIE.STRUCTURE", "CO.VOIE.ORDER"]);
    }

    #[test]
    fn rank_and_ability_convert_from_primitives() {
        assert_eq!(Rank::from(3), Rank::new(3));
        assert_eq!(AbilityId::from("a"), AbilityId::new("a"));
        assert_eq!(AbilityId::from("b".to_owned()), AbilityId::new("b"));
        assert_eq!(VoieId::new("test-voie").as_str(), "test-voie");
    }
}
