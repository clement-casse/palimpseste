//! Voie skeleton: a voie, its ranks and the ability at each rank.

use std::collections::BTreeMap;

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

/// A voie: one ability per rank (CO.VOIE.STRUCTURE).
///
/// Structure only. Gaps, unusual ranks and any number of ranks are
/// representable; costs and acquisition order are engine rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voie {
    id: VoieId,
    abilities: BTreeMap<Rank, AbilityId>,
}

impl Voie {
    /// Creates a voie with no ability.
    #[must_use]
    pub fn new(id: VoieId) -> Self {
        Self {
            id,
            abilities: BTreeMap::new(),
        }
    }

    /// Attaches `ability` to `rank`, replacing any ability already there.
    #[must_use]
    pub fn with_ability(mut self, rank: impl Into<Rank>, ability: impl Into<AbilityId>) -> Self {
        self.abilities.insert(rank.into(), ability.into());
        self
    }

    /// Returns the identifier of the voie.
    #[must_use]
    pub fn id(&self) -> &VoieId {
        &self.id
    }

    /// Returns the ability at `rank`, if any.
    #[must_use]
    pub fn ability_at(&self, rank: Rank) -> Option<&AbilityId> {
        self.abilities.get(&rank)
    }

    /// Iterates over the abilities in increasing rank order.
    pub fn abilities(&self) -> impl Iterator<Item = (Rank, &AbilityId)> {
        self.abilities
            .iter()
            .map(|(rank, ability)| (*rank, ability))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranks(voie: &Voie) -> Vec<u8> {
        voie.abilities().map(|(rank, _)| rank.get()).collect()
    }

    #[test]
    fn abilities_come_back_in_rank_order() {
        let voie = Voie::new(VoieId::new("test-voie"))
            .with_ability(3, "c")
            .with_ability(1, "a")
            .with_ability(2, "b");
        assert_eq!(voie.id().as_str(), "test-voie");
        let ids: Vec<_> = voie.abilities().map(|(_, a)| a.as_str()).collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }

    #[test]
    fn unusual_ranks_and_gaps_are_representable() {
        let gaps = Voie::new(VoieId::new("gaps"))
            .with_ability(0, "zero")
            .with_ability(3, "three");
        assert_eq!(ranks(&gaps), [0, 3]);

        let high = (4..=8).fold(Voie::new(VoieId::new("high")), |voie, rank| {
            voie.with_ability(rank, format!("r{rank}"))
        });
        assert_eq!(ranks(&high), [4, 5, 6, 7, 8]);
    }

    #[test]
    fn same_rank_replaces_the_ability() {
        let voie = Voie::new(VoieId::new("test-voie"))
            .with_ability(1, "old")
            .with_ability(1, "new");
        assert_eq!(voie.ability_at(Rank::new(1)), Some(&AbilityId::new("new")));
        assert_eq!(ranks(&voie), [1]);
    }

    #[test]
    fn rank_and_ability_convert_from_primitives() {
        assert_eq!(Rank::from(3), Rank::new(3));
        assert_eq!(AbilityId::from("a"), AbilityId::new("a"));
        assert_eq!(AbilityId::from("b".to_owned()), AbilityId::new("b"));
        let voie =
            Voie::new(VoieId::new("test-voie")).with_ability(Rank::new(2), AbilityId::new("b"));
        assert_eq!(voie.ability_at(2.into()), Some(&"b".into()));
    }

    #[test]
    fn missing_rank_has_no_ability() {
        let voie = Voie::new(VoieId::new("test-voie")).with_ability(1, "a");
        assert_eq!(voie.ability_at(Rank::new(2)), None);
    }
}
