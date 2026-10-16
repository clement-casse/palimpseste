//! The creation steps in the book order (COC.CRE.STEPS), and the step each
//! violation belongs to.
//!
//! Steps guide the editor: the book allows deferring and revisiting every
//! choice, and so does the engine.

use co_kernel::{ADJUSTMENT_RULE, Category, RuleId, Subject, Violation};

use crate::{
    Value,
    rules::{
        AGE_BRACKET, CAPACITY_POINTS, CARAC_LIMITS, CARAC_LOWERED, CARAC_PHYSICAL, CARAC_POINTS,
        DOMAINE, DOMAINES, EQUIPMENT, MAX_VOIES, SOCIAL_CLASS, TRAIT_POINTS, TRAITS, VOIE_ORDER,
        VOIE_STRUCTURE,
    },
};

/// A step of the creation of a character.
///
/// The book's step 3 (giving the character a body: name, history,
/// convictions) is narrative and has no step. The starting equipment comes
/// with the voies (COC.CRE.EQUIPMENT) and is chosen before the secondary
/// characteristics it changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Step {
    /// Domaine and age (step 1).
    DomaineAndAge,
    /// Voies and ranks (step 2).
    Voies,
    /// Traits distinctifs (step 4).
    Traits,
    /// Characteristic points (step 5).
    Characteristics,
    /// Weapons, protections and items.
    Equipment,
    /// Secondary characteristics (step 6).
    Secondary,
}

/// The step of each rule; the domaine and adjustment notices are refined in
/// [`Step::of`].
const RULE_STEPS: [(RuleId, Step); 15] = [
    (AGE_BRACKET, Step::DomaineAndAge),
    (DOMAINE, Step::DomaineAndAge),
    (VOIE_STRUCTURE, Step::Voies),
    (VOIE_ORDER, Step::Voies),
    (CAPACITY_POINTS, Step::Voies),
    (MAX_VOIES, Step::Voies),
    (DOMAINES, Step::Traits),
    (TRAITS, Step::Traits),
    (TRAIT_POINTS, Step::Traits),
    (SOCIAL_CLASS, Step::Traits),
    (CARAC_POINTS, Step::Characteristics),
    (CARAC_LOWERED, Step::Characteristics),
    (CARAC_PHYSICAL, Step::Characteristics),
    (CARAC_LIMITS, Step::Characteristics),
    (EQUIPMENT, Step::Equipment),
];

impl Step {
    /// Every step, in the book order.
    pub const ALL: [Self; 6] = [
        Self::DomaineAndAge,
        Self::Voies,
        Self::Traits,
        Self::Characteristics,
        Self::Equipment,
        Self::Secondary,
    ];

    /// Returns the stable identifier of the step.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::DomaineAndAge => "domaine-and-age",
            Self::Voies => "voies",
            Self::Traits => "traits",
            Self::Characteristics => "characteristics",
            Self::Equipment => "equipment",
            Self::Secondary => "secondary",
        }
    }

    /// The French display name of the step.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DomaineAndAge => "Domaine et âge",
            Self::Voies => "Voies et rangs",
            Self::Traits => "Traits distinctifs",
            Self::Characteristics => "Caractéristiques",
            Self::Equipment => "Équipement",
            Self::Secondary => "Caractéristiques secondaires",
        }
    }

    /// The step where `violation` is settled. The voies retained from the
    /// domaine are settled with the voies, a missing domaine with the
    /// domaine. An adjustment notice belongs to the step of what it adjusts:
    /// a characteristic or its points, a voie budget, or a secondary value.
    #[must_use]
    pub fn of(violation: &Violation) -> Self {
        let rule = violation.rule();
        if rule == &ADJUSTMENT_RULE {
            return match violation.subject() {
                Subject::Characteristic(_) => Self::Characteristics,
                Subject::Value(id) if *id == Value::CaracPoints.id() => Self::Characteristics,
                Subject::Value(id)
                    if [Value::CapacityPoints, Value::MaxVoies, Value::MaxRank]
                        .iter()
                        .any(|value| *id == value.id()) =>
                {
                    Self::Voies
                }
                _ => Self::Secondary,
            };
        }
        if rule == &DOMAINE && violation.category() != Category::Incomplete {
            return Self::Voies;
        }
        RULE_STEPS
            .iter()
            .find(|(known, _)| known == rule)
            .map_or(Self::Secondary, |(_, step)| *step)
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Adjustment, Author, Characteristic};

    use super::*;
    use crate::{Character, Domaine, Protection, Target, Voie};

    fn steps(character: &Character) -> Vec<(String, Step)> {
        character
            .violations()
            .iter()
            .map(|violation| (violation.rule().as_str().to_owned(), Step::of(violation)))
            .collect()
    }

    #[test]
    fn a_missing_domaine_is_settled_with_the_domaine() {
        let found = steps(&Character::new());
        assert!(found.contains(&("COC.CRE.DOMAINE".to_owned(), Step::DomaineAndAge)));
        assert!(found.contains(&("COC.CRE.AGE-BRACKET".to_owned(), Step::DomaineAndAge)));
    }

    #[test]
    fn voies_retained_from_the_domaine_are_settled_with_the_voies() {
        let character = Character::new()
            .with_age(20)
            .with_domaine(Domaine::Sante)
            .with_rank(Voie::Crime, 1)
            .with_protection(Protection::Casque);
        let found = steps(&character);
        assert!(found.contains(&("COC.CRE.DOMAINE".to_owned(), Step::Voies)));
        assert!(found.contains(&("COC.CRE.EQUIPMENT".to_owned(), Step::Equipment)));
    }

    #[test]
    fn adjustment_notices_follow_their_subject() {
        let adjustment = || Adjustment::new(1, Author::new("gm"), "test");
        let character = Character::new()
            .with_adjustment(Target::Characteristic(Characteristic::For), adjustment())
            .with_adjustment(Target::Value(Value::CaracPoints), adjustment())
            .with_adjustment(Target::Value(Value::MaxRank), adjustment())
            .with_adjustment(Target::Value(Value::Defence), adjustment());
        let notices: Vec<Step> = steps(&character)
            .into_iter()
            .filter(|(rule, _)| rule == ADJUSTMENT_RULE.as_str())
            .map(|(_, step)| step)
            .collect();
        assert_eq!(
            notices,
            [
                Step::Characteristics,
                Step::Characteristics,
                Step::Voies,
                Step::Secondary,
            ]
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
