//! The creation steps in the book order (COF.CRE.STEPS), and the step each
//! violation belongs to.
//!
//! Steps guide the editor: every choice can still be made in any order.

use co_kernel::{ADJUSTMENT_RULE, RuleId, Subject, VOIE_ORDER, VOIE_STRUCTURE, Violation};

use crate::{
    Value,
    rules::{
        ARMOUR_LIMIT, CARAC_BOUNDS, CARAC_KEY, CARAC_MAGIC, CARAC_PEUPLE, CARAC_SERIES, COST,
        EQUIPMENT, HYBRID, LEVEL_REQ, MAGE_EXTRA, MAX_VOIES, NEW_VOIES, ORDER, PEUPLE, PEUPLES, PM,
        POINTS, PROFIL, RANGED, VOIE_DU_MAGE, VOIES,
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
const RULE_STEPS: [(RuleId, Step); 24] = [
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
}
