//! The 18 voies (COC.CNT.VOIES) and the checks on acquired ranks.

use std::collections::BTreeSet;

use co_kernel::{AbilityId, Rank, Severity, Subject, Violation, VoieId};

use crate::{
    AttackMode, Character, Value,
    rules::{CAPACITY_POINTS, MAX_VOIES, VOIE_MECHANIC, VOIE_ORDER, VOIE_STRUCTURE},
};

/// One of the 18 voies of COC2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Voie {
    /// Voie des armes de contact.
    ArmesDeContact,
    /// Voie des armes à distance.
    ArmesADistance,
    /// Voie des armes lourdes.
    ArmesLourdes,
    /// Voie des arts et artisanats.
    ArtsEtArtisanats,
    /// Voie de l'athlétisme.
    Athletisme,
    /// Voie du combat à mains nues.
    CombatAMainsNues,
    /// Voie des corporations.
    Corporations,
    /// Voie du crime.
    Crime,
    /// Voie du discours.
    Discours,
    /// Voie de la furtivité.
    Furtivite,
    /// Voie de l'investigation.
    Investigation,
    /// Voie des langues.
    Langues,
    /// Voie de la médecine.
    Medecine,
    /// Voie de la psychologie.
    Psychologie,
    /// Voie de la technologie.
    Technologie,
    /// Voie des sciences.
    Sciences,
    /// Voie de la survie.
    Survie,
    /// Voie des véhicules.
    Vehicules,
}

impl Voie {
    /// Every voie, in declaration order.
    pub const ALL: [Self; 18] = [
        Self::ArmesDeContact,
        Self::ArmesADistance,
        Self::ArmesLourdes,
        Self::ArtsEtArtisanats,
        Self::Athletisme,
        Self::CombatAMainsNues,
        Self::Corporations,
        Self::Crime,
        Self::Discours,
        Self::Furtivite,
        Self::Investigation,
        Self::Langues,
        Self::Medecine,
        Self::Psychologie,
        Self::Technologie,
        Self::Sciences,
        Self::Survie,
        Self::Vehicules,
    ];

    /// Returns the voie whose [`id`](Self::id) is `id`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|voie| voie.slug() == id)
    }

    /// The French display name of the voie.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ArmesDeContact => "Voie des armes de contact",
            Self::ArmesADistance => "Voie des armes à distance",
            Self::ArmesLourdes => "Voie des armes lourdes",
            Self::ArtsEtArtisanats => "Voie des arts et artisanats",
            Self::Athletisme => "Voie de l'athlétisme",
            Self::CombatAMainsNues => "Voie du combat à mains nues",
            Self::Corporations => "Voie des corporations",
            Self::Crime => "Voie du crime",
            Self::Discours => "Voie du discours",
            Self::Furtivite => "Voie de la furtivité",
            Self::Investigation => "Voie de l'investigation",
            Self::Langues => "Voie des langues",
            Self::Medecine => "Voie de la médecine",
            Self::Psychologie => "Voie de la psychologie",
            Self::Technologie => "Voie de la technologie",
            Self::Sciences => "Voie des sciences",
            Self::Survie => "Voie de la survie",
            Self::Vehicules => "Voie des véhicules",
        }
    }

    /// The weapon category the voie's starting equipment gives one weapon of;
    /// the other voies give free text items only (COC.CNT.VOIES).
    #[must_use]
    pub const fn starting_weapon(self) -> Option<AttackMode> {
        match self {
            Self::ArmesDeContact => Some(AttackMode::ContactWeapon),
            Self::ArmesADistance => Some(AttackMode::RangedWeapon),
            Self::ArmesLourdes => Some(AttackMode::HeavyWeapon),
            _ => None,
        }
    }

    /// Returns the kernel identifier of the voie.
    #[must_use]
    pub fn id(self) -> VoieId {
        VoieId::new(self.slug())
    }

    /// Returns the identifier of the capacité at `rank` in this voie.
    pub(crate) fn ability(self, rank: Rank) -> AbilityId {
        AbilityId::new(format!("{}.{}", self.slug(), rank.get()))
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::ArmesDeContact => "coc.armes-de-contact",
            Self::ArmesADistance => "coc.armes-a-distance",
            Self::ArmesLourdes => "coc.armes-lourdes",
            Self::ArtsEtArtisanats => "coc.arts-et-artisanats",
            Self::Athletisme => "coc.athletisme",
            Self::CombatAMainsNues => "coc.combat-a-mains-nues",
            Self::Corporations => "coc.corporations",
            Self::Crime => "coc.crime",
            Self::Discours => "coc.discours",
            Self::Furtivite => "coc.furtivite",
            Self::Investigation => "coc.investigation",
            Self::Langues => "coc.langues",
            Self::Medecine => "coc.medecine",
            Self::Psychologie => "coc.psychologie",
            Self::Technologie => "coc.technologie",
            Self::Sciences => "coc.sciences",
            Self::Survie => "coc.survie",
            Self::Vehicules => "coc.vehicules",
        }
    }
}

/// Lowest and highest rank of a voie (CO.VOIE.STRUCTURE).
const RANKS: std::ops::RangeInclusive<u8> = 1..=5;

/// Voies allowed in total, whatever the age bracket (COC.CRE.MAX-VOIES).
const ABSOLUTE_MAX_VOIES: i32 = 6;

/// Checks the acquired ranks against the voie structure and the creation
/// budgets.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.voies() {
        check_structure(voie, ranks, violations);
    }
    check_max_rank(character, violations);
    check_voie_budget(character, violations);
    check_max_voies(character, violations);
    check_capacity_points(character, violations);
}

/// CO.VOIE.STRUCTURE and CO.VOIE.ORDER for one voie.
fn check_structure(voie: Voie, ranks: &BTreeSet<Rank>, violations: &mut Vec<Violation>) {
    let subject = || Subject::Voie(voie.id());
    for rank in ranks.iter().filter(|rank| !RANKS.contains(&rank.get())) {
        violations.push(
            Violation::deviation(VOIE_STRUCTURE, VOIE_MECHANIC, subject(), Severity::Standard)
                .with_parameter("rank", i32::from(rank.get())),
        );
    }
    let highest = ranks.last().map_or(0, |rank| rank.get());
    let mut missing = (1..highest)
        .filter(|r| !ranks.contains(&Rank::new(*r)))
        .peekable();
    if let Some(&first) = missing.peek() {
        let count = missing.fold(0_i32, |count, _| count + 1);
        violations.push(
            Violation::deviation(VOIE_ORDER, VOIE_MECHANIC, subject(), Severity::Standard)
                .with_parameter("first_missing", i32::from(first))
                .with_parameter("missing", count),
        );
    }
}

/// COC.CRE.CAPACITY-POINTS: no rank above the bracket's maximum.
fn check_max_rank(character: &Character, violations: &mut Vec<Violation>) {
    let Some(max) = character.value(Value::MaxRank).map(|v| v.total()) else {
        return;
    };
    for (voie, ranks) in character.voies() {
        if let Some(highest) = ranks.last().map(|rank| i32::from(rank.get()))
            && highest > max
        {
            violations.push(
                Violation::deviation(
                    CAPACITY_POINTS,
                    VOIE_MECHANIC,
                    Subject::Voie(voie.id()),
                    Severity::Standard,
                )
                .with_parameter("rank", highest)
                .with_parameter("max", max),
            );
        }
    }
}

/// COC.CRE.CAPACITY-POINTS: no more voies than the bracket allows.
fn check_voie_budget(character: &Character, violations: &mut Vec<Violation>) {
    let count = character.voie_count();
    if let Some(max) = character.value(Value::MaxVoies).map(|v| v.total())
        && count > max
    {
        violations.push(
            Violation::deviation(
                CAPACITY_POINTS,
                VOIE_MECHANIC,
                Subject::Value(Value::MaxVoies.id()),
                Severity::Standard,
            )
            .with_parameter("voies", count)
            .with_parameter("max", max),
        );
    }
}

/// COC.CRE.MAX-VOIES: six voies in total, whatever the bracket.
fn check_max_voies(character: &Character, violations: &mut Vec<Violation>) {
    let count = character.voie_count();
    if count > ABSOLUTE_MAX_VOIES {
        violations.push(
            Violation::deviation(
                MAX_VOIES,
                VOIE_MECHANIC,
                Subject::Character,
                Severity::Standard,
            )
            .with_parameter("voies", count)
            .with_parameter("max", ABSOLUTE_MAX_VOIES),
        );
    }
}

/// COC.CRE.CAPACITY-POINTS: the capacity points spent exactly.
fn check_capacity_points(character: &Character, violations: &mut Vec<Violation>) {
    let Some(budget) = character.value(Value::CapacityPoints).map(|v| v.total()) else {
        return;
    };
    let spent = character.capacity_points_spent();
    let subject = Subject::Value(Value::CapacityPoints.id());
    let violation = match spent.cmp(&budget) {
        std::cmp::Ordering::Less => Violation::incomplete(CAPACITY_POINTS, VOIE_MECHANIC, subject),
        std::cmp::Ordering::Greater => {
            Violation::deviation(CAPACITY_POINTS, VOIE_MECHANIC, subject, Severity::Standard)
        }
        std::cmp::Ordering::Equal => return,
    };
    violations.push(
        violation
            .with_parameter("spent", spent)
            .with_parameter("budget", budget),
    );
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;

    fn voie_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
    }

    fn ranks(character: Character, voie: Voie, highest: u8) -> Character {
        (1..=highest).fold(character, |c, rank| c.with_rank(voie, rank))
    }

    /// A 31-50 character spending exactly its 7 points on 5 voies.
    fn adult() -> Character {
        let character = Character::new().with_age(40);
        let character = ranks(character, Voie::Corporations, 1);
        let character = ranks(character, Voie::ArmesDeContact, 1);
        let character = ranks(character, Voie::Sciences, 1);
        let character = ranks(character, Voie::Survie, 2);
        ranks(character, Voie::Vehicules, 2)
    }

    fn rules(violations: &[Violation]) -> Vec<(&str, Category)> {
        violations
            .iter()
            .map(|v| (v.rule().as_str(), v.category()))
            .collect()
    }

    #[test]
    fn spending_the_budget_within_limits_is_clean() {
        assert_eq!(voie_violations(&adult()), []);
    }

    #[test]
    fn ability_ids_name_voie_and_rank() {
        assert_eq!(Voie::Crime.ability(Rank::new(2)).as_str(), "coc.crime.2");
        assert_eq!(Voie::Survie.id().as_str(), "coc.survie");
    }

    #[test]
    fn unspent_points_are_incomplete() {
        let character = ranks(Character::new().with_age(40), Voie::Survie, 1);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("COC.CRE.CAPACITY-POINTS", Category::Incomplete)]
        );
    }

    #[test]
    fn overspent_points_deviate() {
        let character = ranks(adult(), Voie::Corporations, 2);
        assert_eq!(
            rules(&voie_violations(&character)),
            [(
                "COC.CRE.CAPACITY-POINTS",
                Category::Deviation(Severity::Standard)
            )]
        );
    }

    #[test]
    fn gap_breaks_the_rank_order() {
        let character = Character::new()
            .with_age(20)
            .with_rank(Voie::Crime, 2)
            .with_rank(Voie::Survie, 1)
            .with_rank(Voie::Survie, 2)
            .with_rank(Voie::Langues, 1);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [("CO.VOIE.ORDER", Category::Deviation(Severity::Standard))]
        );
        assert_eq!(violations[0].subject(), &Subject::Voie(Voie::Crime.id()));
    }

    #[test]
    fn several_gaps_in_a_voie_make_one_violation() {
        let character = Character::new().with_rank(Voie::Crime, 200);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [
                ("CO.VOIE.STRUCTURE", Category::Deviation(Severity::Standard)),
                ("CO.VOIE.ORDER", Category::Deviation(Severity::Standard)),
            ]
        );
        let parameters: Vec<_> = violations[1]
            .parameters()
            .iter()
            .map(|p| p.value().clone())
            .collect();
        assert_eq!(
            parameters,
            [
                co_kernel::ParameterValue::Integer(1),
                co_kernel::ParameterValue::Integer(199)
            ]
        );
    }

    #[test]
    fn rank_outside_the_structure_deviates() {
        let character = ranks(Character::new(), Voie::Crime, 5).with_rank(Voie::Crime, 6);
        assert_eq!(
            rules(&voie_violations(&character)),
            [("CO.VOIE.STRUCTURE", Category::Deviation(Severity::Standard))]
        );
    }

    #[test]
    fn rank_above_the_bracket_maximum_deviates() {
        let character =
            ranks(Character::new().with_age(20), Voie::Crime, 3).with_rank(Voie::Survie, 1);
        let violations = voie_violations(&character);
        assert_eq!(
            rules(&violations),
            [(
                "COC.CRE.CAPACITY-POINTS",
                Category::Deviation(Severity::Standard)
            )]
        );
        assert_eq!(violations[0].subject(), &Subject::Voie(Voie::Crime.id()));
    }

    #[test]
    fn too_many_voies_deviate_twice_beyond_six() {
        let voies = [
            Voie::Crime,
            Voie::Discours,
            Voie::Furtivite,
            Voie::Langues,
            Voie::Medecine,
            Voie::Survie,
            Voie::Vehicules,
        ];
        let character = voies
            .into_iter()
            .fold(Character::new(), |c, voie| c.with_rank(voie, 1));
        let character = character.with_age(60);
        let subjects: Vec<_> = voie_violations(&character)
            .into_iter()
            .map(|v| (v.rule().as_str().to_owned(), v.subject().clone()))
            .collect();
        assert_eq!(
            subjects,
            [
                (
                    "COC.CRE.CAPACITY-POINTS".to_owned(),
                    Subject::Value(Value::MaxVoies.id())
                ),
                ("COC.CRE.MAX-VOIES".to_owned(), Subject::Character),
                (
                    "COC.CRE.CAPACITY-POINTS".to_owned(),
                    Subject::Value(Value::CapacityPoints.id())
                ),
            ]
        );
    }

    #[test]
    fn budgets_are_not_checked_without_a_bracket() {
        let character = ranks(Character::new(), Voie::Crime, 4);
        assert_eq!(voie_violations(&character), []);
    }
}
