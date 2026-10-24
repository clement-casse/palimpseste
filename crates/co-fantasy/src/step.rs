//! The creation steps in the book order (COF.CRE.STEPS), the steps of a
//! level-up, and where each violation belongs: a creation step, or a step
//! of the level-up it is about.
//!
//! Steps guide the editor: every choice can still be made in any order.

use co_kernel::{
    ADJUSTMENT_RULE, ParameterValue, Rank, RuleId, Subject, VOIE_ORDER, VOIE_STRUCTURE, Violation,
    voie_rank_violations,
};

use crate::{
    Character, Level, Value, Voie,
    level::LEVEL_PREFIX,
    rules::{
        ARMOUR_LIMIT, CARAC_BOUNDS, CARAC_KEY, CARAC_MAGIC, CARAC_PEUPLE, CARAC_SERIES,
        CONTENT_VOIE, COST, EQUIPMENT, HYBRID, LEVEL_REQ, MAGE_EXTRA, MAX_VOIES, NEW_VOIES, ORDER,
        ORPHAN, PEUPLE, PEUPLES, PM, POINTS, PROFIL, RANGED, REORIENT, VOIE_DU_MAGE, VOIE_MECHANIC,
        VOIES,
    },
};

/// A step of the creation of a level-1 character.
///
/// The book's steps 6 to 14 (PV to DM) compute values from earlier choices:
/// they share one step. The role (step 1) and the final touch (step 15) are
/// narrative and have no step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Step {
    /// Profil, and with it the family (step 2).
    Profil,
    /// Peuple (step 3).
    Peuple,
    /// Value series, its assignment and the peuple modifiers (step 4).
    Characteristics,
    /// Voies and capacités (step 5), with the capacités acquired at each
    /// level-up.
    Voies,
    /// Armour, shield, weapons, items and purse (step 11).
    Equipment,
    /// PV, DR, PC, PM, Init, DEF, attack values and DM (steps 6 to 14).
    Derived,
}

/// The step of each rule; adjustment notices follow their subject.
const RULE_STEPS: [(RuleId, Step); 27] = [
    (PROFIL, Step::Profil),
    (PEUPLE, Step::Peuple),
    (CARAC_SERIES, Step::Characteristics),
    (CARAC_KEY, Step::Characteristics),
    (CARAC_MAGIC, Step::Characteristics),
    (CARAC_PEUPLE, Step::Characteristics),
    (CARAC_BOUNDS, Step::Characteristics),
    (VOIE_STRUCTURE, Step::Voies),
    (VOIE_ORDER, Step::Voies),
    (VOIES, Step::Voies),
    (MAGE_EXTRA, Step::Voies),
    (VOIE_DU_MAGE, Step::Voies),
    (LEVEL_REQ, Step::Voies),
    (POINTS, Step::Voies),
    (COST, Step::Voies),
    (ORDER, Step::Voies),
    (NEW_VOIES, Step::Voies),
    (HYBRID, Step::Voies),
    (MAX_VOIES, Step::Voies),
    (CONTENT_VOIE, Step::Voies),
    (ORPHAN, Step::Voies),
    (REORIENT, Step::Voies),
    (ARMOUR_LIMIT, Step::Equipment),
    (EQUIPMENT, Step::Equipment),
    (RANGED, Step::Equipment),
    (PEUPLES, Step::Equipment),
    (PM, Step::Derived),
];

impl Step {
    /// Every step, in the book order.
    pub const ALL: [Self; 6] = [
        Self::Profil,
        Self::Peuple,
        Self::Characteristics,
        Self::Voies,
        Self::Equipment,
        Self::Derived,
    ];

    /// Returns the stable identifier of the step.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Profil => "profil",
            Self::Peuple => "peuple",
            Self::Characteristics => "characteristics",
            Self::Voies => "voies",
            Self::Equipment => "equipment",
            Self::Derived => "derived",
        }
    }

    /// The French display name of the step.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Profil => "Profil",
            Self::Peuple => "Peuple",
            Self::Characteristics => "Caractéristiques",
            Self::Voies => "Voies et capacités",
            Self::Equipment => "Équipement",
            Self::Derived => "Valeurs dérivées",
        }
    }

    /// The step where `violation` is settled. An adjustment notice belongs
    /// to the step of what it adjusts: a characteristic, the money
    /// (equipment) or a derived value.
    #[must_use]
    pub fn of(violation: &Violation) -> Self {
        if violation.rule() == &ADJUSTMENT_RULE {
            return match violation.subject() {
                Subject::Characteristic(_) => Self::Characteristics,
                Subject::Value(id) if *id == Value::Money.id() => Self::Equipment,
                _ => Self::Derived,
            };
        }
        RULE_STEPS
            .iter()
            .find(|(rule, _)| rule == violation.rule())
            .map_or(Self::Derived, |(_, step)| *step)
    }
}

/// A step of a level-up, in the order its form follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LevelStep {
    /// The capacités acquired and their points (COF.PRG.POINTS, COST,
    /// LEVEL-REQ, ORDER, NEW-VOIES, HYBRID, MAX-VOIES).
    Abilities,
    /// The options of the capacités acquired at the level (COF.CNT.VOIE).
    Options,
    /// The capacités forgotten (COF.PRG.REORIENT).
    Orientation,
    /// The orphan point (COF.PRG.ORPHAN).
    Orphan,
}

impl LevelStep {
    /// Every step of a level-up, in its form's order.
    pub const ALL: [Self; 4] = [
        Self::Abilities,
        Self::Options,
        Self::Orientation,
        Self::Orphan,
    ];

    /// Returns the stable identifier of the step.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Abilities => "abilities",
            Self::Options => "options",
            Self::Orientation => "orientation",
            Self::Orphan => "orphan",
        }
    }

    /// The French display name of the step.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Abilities => "Capacités",
            Self::Options => "Options",
            Self::Orientation => "Changement d'orientation",
            Self::Orphan => "Point orphelin",
        }
    }
}

/// Where a violation is settled: a creation step, or a step of the
/// level-up that reached a level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Placement {
    /// A creation step.
    Creation(Step),
    /// A step of the level-up that reached the level.
    Level(Level, LevelStep),
}

/// The `level` parameter of `violation`.
fn level_parameter(violation: &Violation) -> Option<Level> {
    violation.parameters().iter().find_map(|parameter| {
        match (parameter.name(), parameter.value()) {
            ("level", ParameterValue::Integer(level)) => u8::try_from(*level).ok().map(Level::from),
            _ => None,
        }
    })
}

/// The level whose level-up the violation's subject names (`cof.level.<n>`).
fn level_subject(violation: &Violation) -> Option<Level> {
    match violation.subject() {
        Subject::Value(id) => id
            .as_str()
            .strip_prefix(LEVEL_PREFIX)?
            .parse::<u8>()
            .ok()
            .map(Level::from),
        _ => None,
    }
}

/// Where `violation` is settled (ADR ENG-COF-PLACEMENT): a level-up's
/// findings in a step of that level-up, a capacité option where the
/// capacité was acquired, a voie's structure where it was last changed;
/// anything else in its creation step.
pub(crate) fn placement(character: &Character, violation: &Violation) -> Placement {
    let rule = violation.rule();
    let in_level = |step| {
        level_subject(violation)
            .or_else(|| level_parameter(violation))
            .filter(|level| level.get() > 1)
            .map(|level| Placement::Level(level, step))
    };
    let level = if [POINTS, COST, ORDER, NEW_VOIES, HYBRID, MAX_VOIES].contains(rule)
        || (*rule == LEVEL_REQ && violation.mechanic() != VOIE_MECHANIC)
    {
        in_level(LevelStep::Abilities)
    } else if *rule == REORIENT {
        in_level(LevelStep::Orientation)
    } else if *rule == ORPHAN {
        in_level(LevelStep::Orphan)
    } else if *rule == CONTENT_VOIE {
        match violation.subject() {
            Subject::Ability(id) => Voie::from_ability_id(id.as_str())
                .and_then(|(voie, rank)| character.acquired_at(voie, rank))
                .map(|level| Placement::Level(level, LevelStep::Options)),
            _ => None,
        }
    } else if *rule == VOIE_ORDER || *rule == VOIE_STRUCTURE {
        voie_placement(character, violation)
    } else {
        None
    };
    level.unwrap_or_else(|| Placement::Creation(Step::of(violation)))
}

/// A voie structure finding the creation ranks alone do not raise belongs
/// to the last level-up changing the voie.
fn voie_placement(character: &Character, violation: &Violation) -> Option<Placement> {
    let Subject::Voie(id) = violation.subject() else {
        return None;
    };
    let voie = Voie::from_id(id.as_str())?;
    let created: std::collections::BTreeSet<Rank> = character
        .creation_voies()
        .find(|(created, _)| *created == voie)
        .map(|(_, ranks)| ranks.clone())
        .unwrap_or_default();
    let from_creation = voie_rank_violations(&voie.id(), &created, VOIE_MECHANIC)
        .iter()
        .any(|found| found.rule() == violation.rule());
    if from_creation {
        return None;
    }
    character
        .last_change(voie)
        .map(|level| Placement::Level(level, LevelStep::Abilities))
}

#[cfg(test)]
mod tests {
    use co_kernel::{Adjustment, Author, Characteristic};

    use super::*;
    use crate::{Character, Money, Profil, Target, Voie, Weapon};

    fn steps(character: &Character) -> Vec<(String, Step)> {
        character
            .violations()
            .iter()
            .map(|violation| (violation.rule().as_str().to_owned(), Step::of(violation)))
            .collect()
    }

    #[test]
    fn a_new_character_misses_choices_in_each_step() {
        let found: Vec<Step> = steps(&Character::new())
            .into_iter()
            .map(|(_, step)| step)
            .collect();
        for step in [
            Step::Profil,
            Step::Peuple,
            Step::Characteristics,
            Step::Voies,
            Step::Equipment,
        ] {
            assert!(found.contains(&step), "{step:?}");
        }
    }

    #[test]
    fn rules_map_to_their_step() {
        let character = Character::new()
            .with_profil(Profil::Guerrier)
            .with_rank(Voie::Combat, 2)
            .with_item(Weapon::Mousquet, 1);
        let found = steps(&character);
        assert!(found.contains(&("COF.PRG.LEVEL-REQ".to_owned(), Step::Voies)));
        assert!(found.contains(&("COF.CRE.EQUIPMENT".to_owned(), Step::Equipment)));
    }

    #[test]
    fn adjustment_notices_follow_their_subject() {
        let adjustment = || Adjustment::new(1, Author::new("gm"), "test");
        let character = Character::new()
            .with_adjustment(Target::Characteristic(Characteristic::For), adjustment())
            .with_adjustment(Target::Value(Value::Money), adjustment())
            .with_adjustment(Target::Value(Value::Defence), adjustment())
            .with_purse(Money::pa(7));
        let notices: Vec<Step> = steps(&character)
            .into_iter()
            .filter(|(rule, _)| rule == ADJUSTMENT_RULE.as_str())
            .map(|(_, step)| step)
            .collect();
        assert_eq!(
            notices,
            [Step::Characteristics, Step::Equipment, Step::Derived]
        );
    }

    #[test]
    fn each_rule_has_one_step() {
        for (index, (rule, _)) in RULE_STEPS.iter().enumerate() {
            assert!(
                RULE_STEPS[index + 1..]
                    .iter()
                    .all(|(other, _)| other != rule),
                "{rule:?}"
            );
        }
    }

    /// A humain guerrier at level 1.
    fn guerrier() -> Character {
        Character::new()
            .with_profil(Profil::Guerrier)
            .with_peuple(crate::Peuple::Humain)
            .with_rank(Voie::Bouclier, 1)
            .with_rank(Voie::Soldat, 1)
            .with_rank(Voie::Humain, 1)
    }

    fn placements(character: &Character) -> Vec<(String, Placement)> {
        character
            .violations()
            .iter()
            .map(|v| (v.rule().as_str().to_owned(), character.placement(v)))
            .collect()
    }

    #[test]
    fn level_findings_belong_to_their_level() {
        use crate::{Level, LevelUp, OrphanExchange};
        let two = Level::from(2);
        let character = guerrier().with_level_up(
            LevelUp::new()
                .with_rank(Voie::Soldat, 3)
                .with_forgotten(Voie::Humain, 1)
                .with_orphan(OrphanExchange::LuckPoint),
        );
        let found = placements(&character);
        for (rule, step) in [
            ("COF.PRG.LEVEL-REQ", LevelStep::Abilities),
            ("COF.PRG.REORIENT", LevelStep::Orientation),
            ("COF.PRG.ORPHAN", LevelStep::Orphan),
            // Soldat 3 without rank 2: the level-up made the gap.
            ("CO.VOIE.ORDER", LevelStep::Abilities),
        ] {
            assert!(
                found.contains(&(rule.to_owned(), Placement::Level(two, step))),
                "{rule}: {found:?}"
            );
        }
        // Nothing of the level-up stays in the creation steps.
        assert!(
            found
                .iter()
                .filter(|(rule, _)| rule.starts_with("COF.PRG") || rule.starts_with("CO.VOIE"))
                .all(|(_, placement)| matches!(placement, Placement::Level(..))),
            "{found:?}"
        );
    }

    #[test]
    fn creation_findings_stay_in_their_step() {
        // A rank 2 and a gap at creation.
        let character = guerrier()
            .with_rank(Voie::Soldat, 2)
            .with_rank(Voie::Bouclier, 3)
            .with_level_up(
                crate::LevelUp::new()
                    .with_rank(Voie::Combat, 1)
                    .with_rank(Voie::Combat, 2),
            );
        let found = placements(&character);
        assert!(found.contains(&(
            "COF.PRG.LEVEL-REQ".to_owned(),
            Placement::Creation(Step::Voies)
        )));
        assert!(found.contains(&("CO.VOIE.ORDER".to_owned(), Placement::Creation(Step::Voies))));
    }

    #[test]
    fn an_option_belongs_where_its_capacite_was_acquired() {
        use crate::{Level, LevelUp};
        // Combattant héroïque (rank 4) acquired at level 2, out of rules.
        let character = (1..=3)
            .fold(guerrier(), |c, rank| {
                c.with_rank(Voie::CombatADeuxArmes, rank)
            })
            .with_level_up(LevelUp::new().with_rank(Voie::CombatADeuxArmes, 4));
        let found = placements(&character);
        assert!(found.contains(&(
            "COF.CNT.VOIE".to_owned(),
            Placement::Level(Level::from(2), LevelStep::Options)
        )));
        // Held from creation, it stays in the voies step.
        let created = (1..=4).fold(guerrier(), |c, rank| {
            c.with_rank(Voie::CombatADeuxArmes, rank)
        });
        assert!(
            placements(&created)
                .contains(&("COF.CNT.VOIE".to_owned(), Placement::Creation(Step::Voies)))
        );
    }

    #[test]
    fn level_steps_are_listed_and_named() {
        let ids: Vec<_> = LevelStep::ALL.iter().map(|step| step.id()).collect();
        assert_eq!(ids, ["abilities", "options", "orientation", "orphan"]);
        assert!(LevelStep::ALL.iter().all(|step| !step.name().is_empty()));
    }

    #[test]
    fn a_finding_without_its_usual_subject_stays_in_its_creation_step() {
        use co_kernel::{Mechanic, Severity};
        let character = guerrier();
        let mechanic = Mechanic::new("test");
        let option = Violation::incomplete(CONTENT_VOIE, mechanic, Subject::Character);
        assert_eq!(
            character.placement(&option),
            Placement::Creation(Step::Voies)
        );
        let order =
            Violation::deviation(VOIE_ORDER, mechanic, Subject::Character, Severity::Standard);
        assert_eq!(
            character.placement(&order),
            Placement::Creation(Step::Voies)
        );
    }
}
