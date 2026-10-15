//! Starting characteristics (COC.CRE.CARAC, D-07, D-08, D-10, D-17).

use co_kernel::{
    Characteristic, DerivedValue, Rank, Severity, Source, Subject, ValueId, Violation,
};

use crate::{
    Character, Sequel, Trait, Value, Voie,
    rules::{CARAC_LIMITS, CARAC_LOWERED, CARAC_MECHANIC, CARAC_PHYSICAL, CARAC_POINTS},
};

/// Breakdown source of the characteristic points allocated by the player.
pub const ALLOCATED_POINTS: ValueId = ValueId::from_static("coc.allocated-points");

/// The characteristics whose recommended bounds depend on the age bracket (D-17).
const PHYSICAL: [Characteristic; 3] = [
    Characteristic::For,
    Characteristic::Agi,
    Characteristic::Con,
];

/// Lowest starting value, in every case.
const ABSOLUTE_MIN: i32 = -2;
/// Highest starting value, unless a sequel lowers it.
const ABSOLUTE_MAX: i32 = 5;
/// Highest starting value of the characteristic a Diminué sequel affects.
const DIMINISHED_MAX: i32 = 3;

/// The change `t` makes to the starting value of `c`.
fn trait_modifier(t: Trait, c: Characteristic) -> i32 {
    match (t, c) {
        (Trait::ForceDeLaNature, Characteristic::Con) => 1,
        (Trait::DUnAutreAge, Characteristic::For | Characteristic::Con) => -1,
        (Trait::Sequelle(Sequel::DiminueCorps), Characteristic::Con)
        | (Trait::Sequelle(Sequel::DiminueEsprit), Characteristic::Int) => -2,
        _ => 0,
    }
}

/// The value of `c` before adjustments: allocated points, then trait and
/// capacité modifiers on top (D-08, D-10).
pub(crate) fn characteristic(character: &Character, c: Characteristic) -> DerivedValue {
    let mut value = DerivedValue::new().with(
        Source::Value(ALLOCATED_POINTS),
        character.allocated_points(c),
    );
    for (t, _) in character.traits() {
        let modifier = trait_modifier(t, c);
        if modifier != 0 {
            value = value.with(Source::Value(t.id()), modifier);
        }
    }
    let first_rank = Rank::new(1);
    if c == Characteristic::Con && character.has_rank(Voie::Athletisme, first_rank) {
        value = value.with(Source::Ability(Voie::Athletisme.ability(first_rank)), 1);
    }
    value
}

/// The highest starting value allowed for `c`.
fn absolute_max(character: &Character, c: Characteristic) -> i32 {
    let diminished = match c {
        Characteristic::Con => character.has_trait(Trait::Sequelle(Sequel::DiminueCorps)),
        Characteristic::Int => character.has_trait(Trait::Sequelle(Sequel::DiminueEsprit)),
        _ => false,
    };
    if diminished {
        DIMINISHED_MAX
    } else {
        ABSOLUTE_MAX
    }
}

/// Checks the points spent and the starting values.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    if let Some(budget) = character.value(Value::CaracPoints).map(|v| v.total()) {
        let spent = character.carac_points_spent();
        let subject = Subject::Value(Value::CaracPoints.id());
        let violation = match spent.cmp(&i64::from(budget)) {
            std::cmp::Ordering::Less => {
                Some(Violation::incomplete(CARAC_POINTS, CARAC_MECHANIC, subject))
            }
            std::cmp::Ordering::Greater => Some(Violation::deviation(
                CARAC_POINTS,
                CARAC_MECHANIC,
                subject,
                Severity::Recommended,
            )),
            std::cmp::Ordering::Equal => None,
        };
        if let Some(violation) = violation {
            violations.push(
                violation
                    .with_parameter("spent", spent)
                    .with_parameter("budget", budget),
            );
        }
    }

    let lowered = Characteristic::ALL
        .into_iter()
        .filter(|c| character.allocated_points(*c) < 0)
        .fold(0_i32, |lowered, _| lowered.saturating_add(1));
    if lowered > 1 {
        violations.push(
            Violation::deviation(
                CARAC_LOWERED,
                CARAC_MECHANIC,
                Subject::Character,
                Severity::Recommended,
            )
            .with_parameter("lowered", lowered)
            .with_parameter("max", 1),
        );
    }

    let values = character.characteristics();
    if let Some(limits) = character.age_bracket().map(crate::AgeBracket::limits) {
        for c in PHYSICAL {
            let value = values[c].get();
            if !(limits.physical_min..=limits.physical_max).contains(&value) {
                violations.push(
                    Violation::deviation(
                        CARAC_PHYSICAL,
                        CARAC_MECHANIC,
                        Subject::Characteristic(c),
                        Severity::Recommended,
                    )
                    .with_parameter("value", value)
                    .with_parameter("min", limits.physical_min)
                    .with_parameter("max", limits.physical_max),
                );
            }
        }
    }
    for (c, value) in values.iter() {
        let value = value.get();
        let max = absolute_max(character, c);
        if !(ABSOLUTE_MIN..=max).contains(&value) {
            violations.push(
                Violation::deviation(
                    CARAC_LIMITS,
                    CARAC_MECHANIC,
                    Subject::Characteristic(c),
                    Severity::Absolute,
                )
                .with_parameter("value", value)
                .with_parameter("min", ABSOLUTE_MIN)
                .with_parameter("max", max),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{Adjustment, Author, Category};

    use super::*;
    use crate::Target;

    fn carac_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
    }

    fn rules(violations: &[Violation]) -> Vec<(&str, Category)> {
        violations
            .iter()
            .map(|v| (v.rule().as_str(), v.category()))
            .collect()
    }

    /// A 16-30 character spending its 10 points within every bound.
    fn young() -> Character {
        Character::new()
            .with_age(20)
            .with_carac_points(Characteristic::Agi, 3)
            .with_carac_points(Characteristic::Per, 3)
            .with_carac_points(Characteristic::Int, 2)
            .with_carac_points(Characteristic::Vol, 2)
    }

    #[test]
    fn spending_the_budget_within_bounds_is_clean() {
        assert_eq!(carac_violations(&young()), []);
    }

    #[test]
    fn unspent_points_are_incomplete() {
        let character = young().with_carac_points(Characteristic::Vol, 0);
        assert_eq!(
            rules(&carac_violations(&character)),
            [("COC.CRE.CARAC-POINTS", Category::Incomplete)]
        );
    }

    #[test]
    fn overspent_points_deviate_as_recommended() {
        let character = young().with_carac_points(Characteristic::Cha, 1);
        assert_eq!(
            rules(&carac_violations(&character)),
            [(
                "COC.CRE.CARAC-POINTS",
                Category::Deviation(Severity::Recommended)
            )]
        );
    }

    #[test]
    fn lowering_one_characteristic_recovers_its_points() {
        let character = young()
            .with_carac_points(Characteristic::Cha, -1)
            .with_carac_points(Characteristic::Con, 1);
        assert_eq!(character.carac_points_spent(), 10);
        assert_eq!(carac_violations(&character), []);
    }

    #[test]
    fn lowering_two_characteristics_deviates() {
        let character = Character::new()
            .with_age(20)
            .with_carac_points(Characteristic::Agi, 3)
            .with_carac_points(Characteristic::Per, 4)
            .with_carac_points(Characteristic::Vol, 5)
            .with_carac_points(Characteristic::Cha, -1)
            .with_carac_points(Characteristic::Int, -1);
        assert_eq!(character.carac_points_spent(), 10);
        assert_eq!(
            rules(&carac_violations(&character)),
            [(
                "COC.CRE.CARAC-LOWERED",
                Category::Deviation(Severity::Recommended)
            )]
        );
    }

    #[test]
    fn physical_value_outside_the_bracket_bounds_deviates() {
        let character = Character::new()
            .with_age(40)
            .with_carac_points(Characteristic::For, 5)
            .with_carac_points(Characteristic::Int, 3);
        let violations = carac_violations(&character);
        assert_eq!(
            rules(&violations),
            [(
                "COC.CRE.CARAC-PHYSICAL",
                Category::Deviation(Severity::Recommended)
            )]
        );
        assert_eq!(
            violations[0].subject(),
            &Subject::Characteristic(Characteristic::For)
        );
    }

    #[test]
    fn value_outside_the_absolute_limits_deviates_as_absolute() {
        let character = Character::new()
            .with_carac_points(Characteristic::Int, 6)
            .with_carac_points(Characteristic::Cha, -3);
        let violations = carac_violations(&character);
        assert_eq!(
            rules(&violations),
            [
                (
                    "COC.CRE.CARAC-LIMITS",
                    Category::Deviation(Severity::Absolute)
                ),
                (
                    "COC.CRE.CARAC-LIMITS",
                    Category::Deviation(Severity::Absolute)
                ),
            ]
        );
    }

    #[test]
    fn adjustments_count_towards_the_limits() {
        let adjustment = Adjustment::new(2, Author::new("gm"), "test reason");
        let character = Character::new()
            .with_carac_points(Characteristic::Int, 4)
            .with_adjustment(Target::Characteristic(Characteristic::Int), adjustment);
        assert_eq!(
            rules(&carac_violations(&character)),
            [(
                "COC.CRE.CARAC-LIMITS",
                Category::Deviation(Severity::Absolute)
            )]
        );
    }

    #[test]
    fn diminue_lowers_the_value_and_its_maximum() {
        let character = Character::new()
            .with_trait(Trait::Sequelle(Sequel::DiminueEsprit), 2)
            .with_carac_points(Characteristic::Int, 6);
        assert_eq!(character.characteristics()[Characteristic::Int].get(), 4);
        let violations = carac_violations(&character);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].parameters()[2].value(),
            &co_kernel::ParameterValue::Integer(3)
        );
    }

    #[test]
    fn traits_and_athletisme_modify_the_allocated_points() {
        let character = Character::new()
            .with_carac_points(Characteristic::Con, 1)
            .with_trait(Trait::ForceDeLaNature, 2)
            .with_trait(Trait::DUnAutreAge, 2)
            .with_rank(Voie::Athletisme, 1);
        let con = characteristic(&character, Characteristic::Con);
        let sources: Vec<_> = con
            .contributions()
            .iter()
            .map(|c| (c.source().clone(), c.amount()))
            .collect();
        assert_eq!(
            sources,
            [
                (Source::Value(ALLOCATED_POINTS), 1),
                (Source::Value(Trait::ForceDeLaNature.id()), 1),
                (Source::Value(Trait::DUnAutreAge.id()), -1),
                (Source::Ability(Voie::Athletisme.ability(Rank::new(1))), 1),
            ]
        );
        assert_eq!(con.total(), 2);
        assert_eq!(characteristic(&character, Characteristic::For).total(), -1);
    }
}
