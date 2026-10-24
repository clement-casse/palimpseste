//! The creation steps in the book order (COC.CRE.STEPS), the steps of a
//! spending of XP, and where each violation belongs: a creation step, or a
//! step of the spending it is about.
//!
//! Steps guide the editor: the book allows deferring and revisiting every
//! choice, and so does the engine.

use co_kernel::{
    ADJUSTMENT_RULE, Category, ParameterValue, Rank, RuleId, Subject, VOIE_ORDER, VOIE_STRUCTURE,
    Violation, voie_rank_violations,
};

use crate::{
    Character, Value, Voie,
    rules::{
        AGE_BRACKET, CAPACITY_POINTS, CARAC_GROWTH, CARAC_LIMITS, CARAC_LOWERED, CARAC_PHYSICAL,
        CARAC_POINTS, DOMAINE, DOMAINES, EQUIPMENT, MAX_VOIES, ONE_RANK, PRG_COST, PRG_LIMITS,
        SOCIAL_CLASS, TRAIT_POINTS, TRAITS, VOIE_MECHANIC, XP_USE,
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
const RULE_STEPS: [(RuleId, Step); 20] = [
    (AGE_BRACKET, Step::DomaineAndAge),
    (DOMAINE, Step::DomaineAndAge),
    (VOIE_STRUCTURE, Step::Voies),
    (VOIE_ORDER, Step::Voies),
    (CAPACITY_POINTS, Step::Voies),
    (MAX_VOIES, Step::Voies),
    (PRG_COST, Step::Voies),
    (ONE_RANK, Step::Voies),
    (PRG_LIMITS, Step::Voies),
    (XP_USE, Step::Traits),
    (CARAC_GROWTH, Step::Characteristics),
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

/// A step of a spending of XP, in the order its form follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpendingStep {
    /// The ranks bought and their XP (COC.PRG.COST, ONE-RANK, LIMITS).
    Ranks,
    /// The trait changes bought (COC.PRG.XP-USE).
    Traits,
    /// The characteristic a rank 5 bought raises (COC.PRG.CARAC).
    Characteristics,
}

impl SpendingStep {
    /// Every step of a spending, in its form's order.
    pub const ALL: [Self; 3] = [Self::Ranks, Self::Traits, Self::Characteristics];

    /// Returns the stable identifier of the step.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Ranks => "ranks",
            Self::Traits => "traits",
            Self::Characteristics => "characteristics",
        }
    }

    /// The French display name of the step.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ranks => "Rangs",
            Self::Traits => "Traits",
            Self::Characteristics => "Caractéristiques",
        }
    }
}

/// Where a violation is settled: a creation step, or a step of a spending of
/// XP, numbered from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Placement {
    /// A creation step.
    Creation(Step),
    /// A step of the spending numbered so.
    Spending(usize, SpendingStep),
}

/// The `spending` parameter of `violation`.
fn spending_parameter(violation: &Violation) -> Option<usize> {
    violation.parameters().iter().find_map(|parameter| {
        match (parameter.name(), parameter.value()) {
            ("spending", ParameterValue::Integer(number)) => usize::try_from(*number).ok(),
            _ => None,
        }
    })
}

/// Where `violation` is settled (ADR ENG-COC-PLACEMENT): a spending's
/// findings in a step of that spending, XP spent beyond the XP earned with
/// the last spending, a rank 5's characteristic where the rank was bought, a
/// voie's structure where it was last changed; anything else in its creation
/// step.
pub(crate) fn placement(character: &Character, violation: &Violation) -> Placement {
    let rule = violation.rule();
    let spending = if *rule == PRG_COST
        && violation.subject() == &Subject::Value(Value::Experience.id())
    {
        Some(character.spending_count())
            .filter(|count| *count > 0)
            .map(|count| Placement::Spending(count, SpendingStep::Ranks))
    } else if [PRG_COST, ONE_RANK, PRG_LIMITS].contains(rule) {
        spending_parameter(violation).map(|number| Placement::Spending(number, SpendingStep::Ranks))
    } else if *rule == XP_USE {
        spending_parameter(violation)
            .map(|number| Placement::Spending(number, SpendingStep::Traits))
    } else if *rule == CARAC_GROWTH {
        match violation.subject() {
            Subject::Ability(id) => Voie::from_ability_id(id.as_str())
                .and_then(|(voie, rank)| character.bought_in(voie, rank))
                .map(|number| Placement::Spending(number, SpendingStep::Characteristics)),
            _ => None,
        }
    } else if *rule == VOIE_ORDER || *rule == VOIE_STRUCTURE {
        voie_placement(character, violation)
    } else {
        None
    };
    spending.unwrap_or_else(|| Placement::Creation(Step::of(violation)))
}

/// A voie structure finding the creation ranks alone do not raise belongs
/// to the last spending buying a rank of the voie.
fn voie_placement(character: &Character, violation: &Violation) -> Option<Placement> {
    let Subject::Voie(id) = violation.subject() else {
        return None;
    };
    let voie = Voie::from_id(id.as_str())?;
    let created: std::collections::BTreeSet<Rank> = character
        .creation_ranks()
        .get(&voie)
        .cloned()
        .unwrap_or_default();
    let from_creation = voie_rank_violations(&voie.id(), &created, VOIE_MECHANIC)
        .iter()
        .any(|found| found.rule() == violation.rule());
    if from_creation {
        return None;
    }
    character
        .last_spending_on(voie)
        .map(|number| Placement::Spending(number, SpendingStep::Ranks))
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

    fn placements(character: &Character) -> Vec<(String, Placement)> {
        character
            .violations()
            .iter()
            .map(|v| (v.rule().as_str().to_owned(), character.placement(v)))
            .collect()
    }

    #[test]
    fn spending_findings_belong_to_their_spending() {
        use crate::{Trait, XpSpending};
        let character = Character::new()
            .with_rank(Voie::Crime, 1)
            .with_experience(1)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Medecine, 1))
            .with_xp_spending(
                XpSpending::new()
                    .with_rank(Voie::Crime, 3)
                    .with_rank(Voie::Crime, 4)
                    .with_trait(Trait::Rapide, 2),
            );
        let found = placements(&character);
        for (rule, placement) in [
            (
                "COC.PRG.ONE-RANK",
                Placement::Spending(2, SpendingStep::Ranks),
            ),
            (
                "COC.PRG.XP-USE",
                Placement::Spending(2, SpendingStep::Traits),
            ),
            ("COC.PRG.COST", Placement::Spending(2, SpendingStep::Ranks)),
            ("CO.VOIE.ORDER", Placement::Spending(2, SpendingStep::Ranks)),
        ] {
            assert!(
                found.contains(&(rule.to_owned(), placement)),
                "{rule}: {found:?}"
            );
        }
    }

    #[test]
    fn a_rank_five_choice_belongs_where_the_rank_was_bought() {
        use crate::XpSpending;
        let bought = (1..=4)
            .fold(Character::new(), |c, rank| c.with_rank(Voie::Crime, rank))
            .with_experience(5)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Crime, 5));
        assert!(placements(&bought).contains(&(
            "COC.PRG.CARAC".to_owned(),
            Placement::Spending(1, SpendingStep::Characteristics)
        )));
        let created = (1..=5).fold(Character::new(), |c, rank| c.with_rank(Voie::Crime, rank));
        assert!(placements(&created).contains(&(
            "COC.PRG.CARAC".to_owned(),
            Placement::Creation(Step::Characteristics)
        )));
    }

    #[test]
    fn spending_steps_are_listed_and_named() {
        let ids: Vec<_> = SpendingStep::ALL.iter().map(|step| step.id()).collect();
        assert_eq!(ids, ["ranks", "traits", "characteristics"]);
        assert!(SpendingStep::ALL.iter().all(|step| !step.name().is_empty()));
    }

    #[test]
    fn a_seventh_voie_belongs_to_the_spending_opening_it() {
        use crate::XpSpending;
        let character = [
            Voie::Crime,
            Voie::Discours,
            Voie::Furtivite,
            Voie::Langues,
            Voie::Medecine,
            Voie::Sciences,
        ]
        .into_iter()
        .fold(Character::new(), |c, voie| c.with_rank(voie, 1))
        .with_experience(1)
        .with_xp_spending(XpSpending::new().with_rank(Voie::Survie, 1));
        assert!(placements(&character).contains(&(
            "COC.PRG.LIMITS".to_owned(),
            Placement::Spending(1, SpendingStep::Ranks)
        )));
    }

    #[test]
    fn a_gap_left_at_creation_stays_in_the_voies_step() {
        use crate::XpSpending;
        let character = Character::new()
            .with_rank(Voie::Crime, 1)
            .with_rank(Voie::Crime, 3)
            .with_experience(1)
            .with_xp_spending(XpSpending::new().with_rank(Voie::Medecine, 1));
        assert!(
            placements(&character)
                .contains(&("CO.VOIE.ORDER".to_owned(), Placement::Creation(Step::Voies)))
        );
    }

    #[test]
    fn a_finding_without_its_usual_subject_stays_in_its_creation_step() {
        use co_kernel::{Mechanic, Severity, Violation};

        use crate::rules::CARAC_GROWTH;
        let character = Character::new();
        let mechanic = Mechanic::new("test");
        let growth = Violation::incomplete(CARAC_GROWTH, mechanic, Subject::Character);
        assert_eq!(
            character.placement(&growth),
            Placement::Creation(Step::Characteristics)
        );
        let order =
            Violation::deviation(VOIE_ORDER, mechanic, Subject::Character, Severity::Standard);
        assert_eq!(
            character.placement(&order),
            Placement::Creation(Step::Voies)
        );
    }
}
