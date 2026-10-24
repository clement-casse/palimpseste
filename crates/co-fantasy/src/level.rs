//! Levels after the first (COF.PRG.*): the capacités acquired at each
//! level-up, and the checks on them.
//!
//! A character stores its level-ups in order (ADR ENG-COF-PROGRESSION); its
//! level, its current ranks and the values they feed are computed from them.

use std::collections::{BTreeMap, BTreeSet};

use co_kernel::{Die, Rank, Severity, Subject, ValueId, Violation};

use crate::{
    Character, Voie,
    rules::{
        COST, HYBRID, LEVEL_REQ, MAX_VOIES, NEW_VOIES, ORDER, PEUPLE_VOIE, POINTS, PROFIL_VOIES,
        PROGRESSION_MECHANIC,
    },
};

/// Capacity points gained at each level-up (COF.PRG.POINTS).
const POINTS_PER_LEVEL: i32 = 2;

/// Voies a character may hold besides the voie de peuple (COF.PRG.MAX-VOIES).
const MAX_PROFIL_VOIES: i32 = 6;

/// The highest level whose attacks still add the level (COF.DER.ATTACK).
const ATTACK_LEVEL_CAP: u8 = 10;

/// A character's level, 1 at creation (COF.CRE.LEVEL1) and 1 more per
/// level-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Level(u8);

impl Level {
    /// Returns the level as a number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The die a dé évolutif d4° stands for at this level (COF.DER.D4).
    #[must_use]
    pub const fn evolutive_die(self) -> Die {
        Die::new(match self.0 {
            0..=5 => 4,
            6..=8 => 6,
            9..=11 => 8,
            12..=14 => 10,
            _ => 12,
        })
    }

    /// The level term of the attack values: the level, at most 10
    /// (COF.DER.ATTACK).
    pub(crate) fn attack_term(self) -> i32 {
        i32::from(self.0.min(ATTACK_LEVEL_CAP))
    }

    /// The kernel identifier naming the level-up that reached this level.
    pub(crate) fn id(self) -> ValueId {
        ValueId::new(format!("{LEVEL_PREFIX}{}", self.0))
    }
}

impl From<u8> for Level {
    fn from(level: u8) -> Self {
        Self(level)
    }
}

/// The prefix of a level-up identifier, followed by the level reached.
pub(crate) const LEVEL_PREFIX: &str = "cof.level.";

/// The capacités acquired at one level-up, with the capacity points gained
/// at that level (COF.PRG.LEVEL-UP).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LevelUp {
    ranks: BTreeSet<(Voie, Rank)>,
}

impl LevelUp {
    /// Creates a level-up acquiring no capacité.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquires `rank` in `voie` at this level.
    #[must_use]
    pub fn with_rank(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.ranks.insert((voie, rank.into()));
        self
    }

    /// Gives up `rank` in `voie` at this level.
    #[must_use]
    pub fn without_rank(mut self, voie: Voie, rank: impl Into<Rank>) -> Self {
        self.ranks.remove(&(voie, rank.into()));
        self
    }

    /// The ranks acquired at this level, in voie and rank order.
    pub(crate) fn ranks(&self) -> impl Iterator<Item = (Voie, Rank)> + '_ {
        self.ranks.iter().copied()
    }

    /// The capacity points the ranks cost (COF.PRG.COST).
    fn points_spent(&self) -> i32 {
        self.ranks().map(|(_, rank)| cost(rank)).sum()
    }
}

/// The capacity points `rank` costs: 1 up to rank 2, then 2 (COF.PRG.COST).
const fn cost(rank: Rank) -> i32 {
    if rank.get() <= 2 { 1 } else { 2 }
}

/// The level required to acquire `rank` (COF.PRG.LEVEL-REQ), if the table
/// gives one. Ranks 6 to 8 belong to prestige voies, out of scope.
pub(crate) const fn required_level(rank: Rank) -> Option<i32> {
    match rank.get() {
        1 => Some(1),
        2 => Some(2),
        3 => Some(3),
        4 => Some(5),
        5 => Some(7),
        _ => None,
    }
}

/// Checks the level-ups: points spent, level required, order of the ranks,
/// voies opened and voie count.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    // The ranks held at the end of the previous level, creation included.
    let mut held: BTreeMap<Voie, BTreeSet<Rank>> = character
        .creation_voies()
        .map(|(voie, ranks)| (voie, ranks.clone()))
        .collect();
    let current = character.voies();
    let mut opened_peuple_slot = false;
    for (level, level_up) in character.levels() {
        check_points(level, level_up, violations);
        for (voie, rank) in level_up.ranks() {
            let ability = Subject::Ability(voie.ability(rank));
            if held.get(&voie).is_some_and(|ranks| ranks.contains(&rank)) {
                violations.push(
                    Violation::deviation(
                        COST,
                        PROGRESSION_MECHANIC,
                        ability.clone(),
                        Severity::Standard,
                    )
                    .with_parameter("level", i32::from(level.get())),
                );
            }
            check_required_level(level, rank, ability.clone(), violations);
            if missing_below(level_up, &held, &current, voie, rank) {
                violations.push(
                    Violation::deviation(ORDER, PROGRESSION_MECHANIC, ability, Severity::Standard)
                        .with_parameter("level", i32::from(level.get())),
                );
            }
            if !held.contains_key(&voie) {
                opened_peuple_slot |= voie.is_peuple_slot();
                check_opened(character, level, voie, violations);
            }
        }
        for (voie, rank) in level_up.ranks() {
            held.entry(voie).or_default().insert(rank);
        }
    }
    check_voie_count(&current, opened_peuple_slot, violations);
}

/// COF.PRG.POINTS: the 2 points of a level are spent at once, no more.
fn check_points(level: Level, level_up: &LevelUp, violations: &mut Vec<Violation>) {
    let spent = level_up.points_spent();
    let subject = Subject::Value(level.id());
    let violation = match spent.cmp(&POINTS_PER_LEVEL) {
        std::cmp::Ordering::Less => Violation::incomplete(POINTS, PROGRESSION_MECHANIC, subject),
        std::cmp::Ordering::Greater => {
            Violation::deviation(POINTS, PROGRESSION_MECHANIC, subject, Severity::Standard)
        }
        std::cmp::Ordering::Equal => return,
    };
    violations.push(
        violation
            .with_parameter("spent", spent)
            .with_parameter("expected", POINTS_PER_LEVEL),
    );
}

/// COF.PRG.LEVEL-REQ: the rank needs the level it was acquired at.
fn check_required_level(
    level: Level,
    rank: Rank,
    ability: Subject,
    violations: &mut Vec<Violation>,
) {
    let required = required_level(rank);
    if required.is_some_and(|required| required <= i32::from(level.get())) {
        return;
    }
    let mut violation =
        Violation::deviation(LEVEL_REQ, PROGRESSION_MECHANIC, ability, Severity::Standard)
            .with_parameter("rank", i32::from(rank.get()))
            .with_parameter("level", i32::from(level.get()));
    if let Some(required) = required {
        violation = violation.with_parameter("required", required);
    }
    violations.push(violation);
}

/// COF.PRG.ORDER: whether the rank below `rank` was acquired only at a later
/// level. A rank below never acquired is CO.VOIE.ORDER's, reported on the
/// current ranks.
fn missing_below(
    level_up: &LevelUp,
    held: &BTreeMap<Voie, BTreeSet<Rank>>,
    current: &BTreeMap<Voie, BTreeSet<Rank>>,
    voie: Voie,
    rank: Rank,
) -> bool {
    let Some(below) = rank.get().checked_sub(1).filter(|below| *below >= 1) else {
        return false;
    };
    let below = Rank::new(below);
    let held_by_then = held.get(&voie).is_some_and(|ranks| ranks.contains(&below))
        || level_up.ranks.contains(&(voie, below));
    let acquired_later = current
        .get(&voie)
        .is_some_and(|ranks| ranks.contains(&below));
    !held_by_then && acquired_later
}

/// COF.PRG.NEW-VOIES and COF.PRG.HYBRID: a voie opened after level 1 is a
/// voie of the main profil. Another profil's makes a hybrid (out of scope),
/// another peuple's or the voie du mage outside the mage family is not
/// offered at all.
fn check_opened(character: &Character, level: Level, voie: Voie, violations: &mut Vec<Violation>) {
    let Some(profil) = character.profil() else {
        return;
    };
    let rule = if voie.is_peuple_slot() {
        if character.peuple_slot_options().contains(&voie) {
            return;
        }
        NEW_VOIES
    } else if voie.profil() == Some(profil) {
        return;
    } else {
        HYBRID
    };
    violations.push(
        Violation::deviation(
            rule,
            PROGRESSION_MECHANIC,
            Subject::Voie(voie.id()),
            Severity::Standard,
        )
        .with_parameter("level", i32::from(level.get())),
    );
}

/// COF.PRG.MAX-VOIES: six voies at most, plus one voie de peuple. A second
/// voie de peuple at creation is COF.CRE.VOIES's, so the slot is checked
/// only when a level-up opens one.
fn check_voie_count(
    current: &BTreeMap<Voie, BTreeSet<Rank>>,
    opened_peuple_slot: bool,
    violations: &mut Vec<Violation>,
) {
    let peuple_slot = current.keys().filter(|voie| voie.is_peuple_slot()).count();
    let profil_voies = current.len() - peuple_slot;
    let count = |count: usize| i32::try_from(count).unwrap_or(i32::MAX);
    if count(profil_voies) > MAX_PROFIL_VOIES {
        violations.push(
            Violation::deviation(
                MAX_VOIES,
                PROGRESSION_MECHANIC,
                Subject::Value(PROFIL_VOIES),
                Severity::Standard,
            )
            .with_parameter("voies", count(profil_voies))
            .with_parameter("max", MAX_PROFIL_VOIES),
        );
    }
    if opened_peuple_slot && peuple_slot > 1 {
        violations.push(
            Violation::deviation(
                MAX_VOIES,
                PROGRESSION_MECHANIC,
                Subject::Value(PEUPLE_VOIE),
                Severity::Standard,
            )
            .with_parameter("voies", count(peuple_slot))
            .with_parameter("max", 1),
        );
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Category, Characteristic};

    use super::*;
    use crate::{AttackMode, Peuple, Profil, Value};

    const DEVIATION: Category = Category::Deviation(Severity::Standard);

    fn level_violations(character: &Character) -> Vec<(String, Subject, Category)> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
            .into_iter()
            .map(|v| {
                (
                    v.rule().as_str().to_owned(),
                    v.subject().clone(),
                    v.category(),
                )
            })
            .collect()
    }

    fn ability(voie: Voie, rank: u8) -> Subject {
        Subject::Ability(voie.ability(Rank::new(rank)))
    }

    fn level(level: u8) -> Subject {
        Subject::Value(Level::from(level).id())
    }

    /// A humain guerrier at level 1.
    fn guerrier() -> Character {
        Character::new()
            .with_profil(Profil::Guerrier)
            .with_peuple(Peuple::Humain)
            .with_rank(Voie::Bouclier, 1)
            .with_rank(Voie::Soldat, 1)
            .with_rank(Voie::Humain, 1)
    }

    /// A level-2 level-up spending its 2 points on two rank 2s.
    fn two_points() -> LevelUp {
        LevelUp::new()
            .with_rank(Voie::Bouclier, 2)
            .with_rank(Voie::Soldat, 2)
    }

    #[test]
    fn level_counts_the_level_ups() {
        assert_eq!(Character::new().level(), Level::from(1));
        let character = guerrier()
            .with_level_up(two_points())
            .with_level_up(LevelUp::new());
        assert_eq!(character.level(), Level::from(3));
        assert_eq!(character.without_level_up().level(), Level::from(2));
    }

    #[test]
    fn evolutive_die_grows_with_the_level() {
        let dice: Vec<u8> = [1, 5, 6, 8, 9, 11, 12, 14, 15, 20]
            .map(|level| Level::from(level).evolutive_die().faces())
            .to_vec();
        assert_eq!(dice, [4, 4, 6, 6, 8, 8, 10, 10, 12, 12]);
    }

    #[test]
    fn attacks_add_the_level_up_to_10() {
        let character = (0..11).fold(guerrier(), |c, _| c.with_level_up(LevelUp::new()));
        assert_eq!(character.level(), Level::from(12));
        let contact = character.value(Value::Attack(AttackMode::Contact));
        assert_eq!(contact.map(|v| v.total()), Some(10));
    }

    #[test]
    fn vigour_counts_the_current_con_for_every_level() {
        let at = |con: i32| {
            let character = (0..7).fold(
                guerrier().with_series_value(Characteristic::Con, con),
                |c, _| c.with_level_up(LevelUp::new()),
            );
            character.value(Value::VigourPoints).map(|v| v.total())
        };
        // Level 8: 9 x 5 (combattants) + 8 x CON.
        assert_eq!(at(2), Some(61));
        // One more CON adds 8 PV.
        assert_eq!(at(3), Some(69));
    }

    #[test]
    fn two_points_spent_are_clean() {
        let character = guerrier()
            .with_level_up(two_points())
            .with_level_up(LevelUp::new().with_rank(Voie::Bouclier, 3));
        assert_eq!(level_violations(&character), []);
    }

    #[test]
    fn points_left_are_incomplete_and_extra_points_deviate() {
        let character = guerrier()
            .with_level_up(LevelUp::new().with_rank(Voie::Bouclier, 2))
            .with_level_up(
                LevelUp::new()
                    .with_rank(Voie::Soldat, 2)
                    .with_rank(Voie::Bouclier, 3),
            );
        assert_eq!(
            level_violations(&character),
            [
                ("COF.PRG.POINTS".to_owned(), level(2), Category::Incomplete),
                ("COF.PRG.POINTS".to_owned(), level(3), DEVIATION),
            ]
        );
    }

    #[test]
    fn rank_needs_its_level_when_acquired() {
        // Rank 3 at level 2, rank 4 at level 3 (it needs level 5).
        let character = guerrier()
            .with_level_up(
                LevelUp::new()
                    .with_rank(Voie::Bouclier, 2)
                    .with_rank(Voie::Bouclier, 3),
            )
            .with_level_up(LevelUp::new().with_rank(Voie::Bouclier, 4));
        let rules: Vec<_> = level_violations(&character)
            .into_iter()
            .filter(|(rule, _, _)| rule == "COF.PRG.LEVEL-REQ")
            .map(|(_, subject, _)| subject)
            .collect();
        assert_eq!(
            rules,
            [ability(Voie::Bouclier, 3), ability(Voie::Bouclier, 4)]
        );
        // A prestige rank never meets the table.
        let prestige = guerrier().with_level_up(LevelUp::new().with_rank(Voie::Bouclier, 6));
        assert!(level_violations(&prestige).contains(&(
            "COF.PRG.LEVEL-REQ".to_owned(),
            ability(Voie::Bouclier, 6),
            DEVIATION
        )));
    }

    #[test]
    fn rank_bought_again_deviates() {
        let character = guerrier().with_level_up(
            LevelUp::new()
                .with_rank(Voie::Soldat, 1)
                .with_rank(Voie::Bouclier, 2),
        );
        assert_eq!(
            level_violations(&character),
            [(
                "COF.PRG.COST".to_owned(),
                ability(Voie::Soldat, 1),
                DEVIATION
            )]
        );
    }

    #[test]
    fn rank_before_the_one_below_deviates() {
        // Rank 3 at level 3, rank 2 only at level 4.
        let character = guerrier()
            .with_level_up(two_points())
            .with_level_up(
                LevelUp::new()
                    .with_rank(Voie::Combat, 1)
                    .with_rank(Voie::Combat, 3),
            )
            .with_level_up(
                LevelUp::new()
                    .with_rank(Voie::Combat, 2)
                    .with_rank(Voie::Resistance, 1),
            );
        assert_eq!(
            level_violations(&character),
            [
                ("COF.PRG.POINTS".to_owned(), level(3), DEVIATION),
                (
                    "COF.PRG.ORDER".to_owned(),
                    ability(Voie::Combat, 3),
                    DEVIATION
                ),
            ]
        );
        // A rank below never acquired is the kernel's CO.VOIE.ORDER.
        let gap = guerrier().with_level_up(LevelUp::new().with_rank(Voie::Soldat, 3));
        let rules: Vec<_> = gap
            .violations()
            .into_iter()
            .map(|v| v.rule().as_str().to_owned())
            .collect();
        assert!(rules.contains(&"CO.VOIE.ORDER".to_owned()));
        assert!(!rules.contains(&"COF.PRG.ORDER".to_owned()));
    }

    #[test]
    fn voies_opened_come_from_the_main_profil() {
        let character = guerrier().with_level_up(
            LevelUp::new()
                .with_rank(Voie::Combat, 1)
                .with_rank(Voie::Rage, 1),
        );
        assert_eq!(
            level_violations(&character),
            [(
                "COF.PRG.HYBRID".to_owned(),
                Subject::Voie(Voie::Rage.id()),
                DEVIATION
            )]
        );
        let mage = guerrier().with_level_up(
            LevelUp::new()
                .with_rank(Voie::Combat, 1)
                .with_rank(Voie::Mage, 1),
        );
        assert_eq!(
            level_violations(&mage),
            [
                (
                    "COF.PRG.NEW-VOIES".to_owned(),
                    Subject::Voie(Voie::Mage.id()),
                    DEVIATION
                ),
                (
                    "COF.PRG.MAX-VOIES".to_owned(),
                    Subject::Value(PEUPLE_VOIE),
                    DEVIATION
                ),
            ]
        );
    }

    #[test]
    fn a_seventh_voie_deviates() {
        // Five guerrier voies, then two hybrid ones.
        let character = guerrier()
            .with_level_up(
                LevelUp::new()
                    .with_rank(Voie::Combat, 1)
                    .with_rank(Voie::MaitreDArmes, 1),
            )
            .with_level_up(
                LevelUp::new()
                    .with_rank(Voie::Resistance, 1)
                    .with_rank(Voie::Rage, 1),
            )
            .with_level_up(
                LevelUp::new()
                    .with_rank(Voie::Brute, 1)
                    .with_rank(Voie::Bouclier, 2),
            );
        let found: Vec<_> = level_violations(&character)
            .into_iter()
            .filter(|(rule, _, _)| rule == "COF.PRG.MAX-VOIES")
            .collect();
        assert_eq!(
            found,
            [(
                "COF.PRG.MAX-VOIES".to_owned(),
                Subject::Value(PROFIL_VOIES),
                DEVIATION
            )]
        );
    }

    #[test]
    fn creation_rules_read_the_creation_ranks_only() {
        // The ranks of a level-up are not level-1 choices: no COF.CRE.VOIES
        // or LEVEL-REQ finding at creation.
        let character = guerrier().with_level_up(two_points());
        let rules: Vec<_> = character
            .violations()
            .into_iter()
            .map(|v| v.rule().as_str().to_owned())
            .filter(|rule| rule.starts_with("COF.CRE.VOIES") || rule == "COF.PRG.LEVEL-REQ")
            .collect();
        assert_eq!(rules, Vec::<String>::new());
    }
}
