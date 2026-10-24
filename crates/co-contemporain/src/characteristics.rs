//! Characteristics (COC.CRE.CARAC, COC.PRG.CARAC, D-07, D-08, D-10, D-17,
//! D-20).

use co_kernel::{
    Characteristic, DerivedValue, Rank, Severity, Source, Subject, ValueId, Violation,
};

use crate::{
    Character, Sequel, Trait, Value, Voie,
    rules::{
        CARAC_GROWTH, CARAC_LIMITS, CARAC_LOWERED, CARAC_MECHANIC, CARAC_PHYSICAL, CARAC_POINTS,
    },
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

/// The choices a characteristic is computed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// The creation choices only: the starting value the creation limits
    /// read.
    Starting,
    /// The creation choices and the XP spendings.
    Current,
}

/// The rank whose capacité raises a characteristic (COC.PRG.CARAC).
const RAISING_RANK: Rank = Rank::new(5);

/// The value of `c` before adjustments: allocated points, then trait and
/// capacité modifiers on top (D-08, D-10): Athlétisme rank 1 and the
/// characteristic each rank 5 raises (D-20).
pub(crate) fn characteristic(
    character: &Character,
    c: Characteristic,
    scope: Scope,
) -> DerivedValue {
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
    let voies = match scope {
        Scope::Starting => character.creation_ranks().clone(),
        Scope::Current => character.voies(),
    };
    let first_rank = Rank::new(1);
    if c == Characteristic::Con
        && voies
            .get(&Voie::Athletisme)
            .is_some_and(|ranks| ranks.contains(&first_rank))
    {
        value = value.with(Source::Ability(Voie::Athletisme.ability(first_rank)), 1);
    }
    for (voie, ranks) in &voies {
        let raised = voie.rank_five_raises().contains(&c)
            || character.raised_characteristic(*voie) == Some(c);
        if ranks.contains(&RAISING_RANK) && raised {
            value = value.with(Source::Ability(voie.ability(RAISING_RANK)), 1);
        }
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
    check_points(character, violations);
    check_lowered(character, violations);
    check_physical(character, violations);
    check_limits(character, violations);
    check_rank_five(character, violations);
}

/// COC.PRG.CARAC: a rank 5 offering a choice has the characteristic it
/// raises chosen among its options.
fn check_rank_five(character: &Character, violations: &mut Vec<Violation>) {
    for (voie, ranks) in character.voies() {
        if !ranks.contains(&RAISING_RANK) {
            continue;
        }
        let choice = voie.rank_five_choice();
        let subject = Subject::Ability(voie.ability(RAISING_RANK));
        match character.raised_characteristic(voie) {
            None if !choice.is_empty() => {
                violations.push(Violation::incomplete(CARAC_GROWTH, CARAC_MECHANIC, subject));
            }
            Some(c) if !choice.contains(&c) => violations.push(Violation::deviation(
                CARAC_GROWTH,
                CARAC_MECHANIC,
                subject,
                Severity::Standard,
            )),
            _ => {}
        }
    }
}

/// COC.CRE.CARAC-POINTS: the characteristic points spent exactly.
fn check_points(character: &Character, violations: &mut Vec<Violation>) {
    let Some(budget) = character.value(Value::CaracPoints).map(|v| v.total()) else {
        return;
    };
    let spent = character.carac_points_spent();
    let subject = Subject::Value(Value::CaracPoints.id());
    let violation = match spent.cmp(&i64::from(budget)) {
        std::cmp::Ordering::Less => Violation::incomplete(CARAC_POINTS, CARAC_MECHANIC, subject),
        std::cmp::Ordering::Greater => {
            Violation::deviation(CARAC_POINTS, CARAC_MECHANIC, subject, Severity::Recommended)
        }
        std::cmp::Ordering::Equal => return,
    };
    violations.push(
        violation
            .with_parameter("spent", spent)
            .with_parameter("budget", budget),
    );
}

/// COC.CRE.CARAC-LOWERED (recommended): one characteristic lowered at most.
fn check_lowered(character: &Character, violations: &mut Vec<Violation>) {
    let lowered = Characteristic::ALL
        .into_iter()
        .filter(|c| character.allocated_points(*c) < 0)
        .fold(0_i32, |lowered, _| lowered + 1);
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
}

/// COC.CRE.CARAC-PHYSICAL (recommended): FOR, AGI and CON within the
/// bracket's bounds (D-17).
fn check_physical(character: &Character, violations: &mut Vec<Violation>) {
    let Some(limits) = character.age_bracket().map(crate::AgeBracket::limits) else {
        return;
    };
    let values = character.starting_characteristics();
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

/// COC.CRE.CARAC-LIMITS (absolute): every starting value within its limits.
fn check_limits(character: &Character, violations: &mut Vec<Violation>) {
    for (c, value) in character.starting_characteristics().iter() {
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
        let con = characteristic(&character, Characteristic::Con, Scope::Current);
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
        assert_eq!(
            characteristic(&character, Characteristic::For, Scope::Current).total(),
            -1
        );
    }

    #[test]
    fn rank_five_raises_a_characteristic() {
        let ranks = |c: Character, voie| (1..=5).fold(c, |c, rank| c.with_rank(voie, rank));
        // Langues raises INT without a choice; Corporations raises two.
        let linguist = ranks(Character::new(), Voie::Langues);
        assert_eq!(linguist.characteristics()[Characteristic::Int].get(), 1);
        let boss = ranks(Character::new(), Voie::Corporations);
        assert_eq!(boss.characteristics()[Characteristic::Cha].get(), 1);
        assert_eq!(boss.characteristics()[Characteristic::Int].get(), 1);
        assert_eq!(carac_violations(&boss), []);
        // Crime offers PER or CHA: incomplete until chosen.
        let criminal = ranks(Character::new(), Voie::Crime);
        let growth = |c: &Character| -> Vec<Category> {
            carac_violations(c)
                .into_iter()
                .filter(|v| v.rule() == &CARAC_GROWTH)
                .map(|v| v.category())
                .collect()
        };
        assert_eq!(growth(&criminal), [Category::Incomplete]);
        let chosen = criminal.with_raised_characteristic(Voie::Crime, Characteristic::Cha);
        assert_eq!(growth(&chosen), []);
        assert_eq!(chosen.characteristics()[Characteristic::Cha].get(), 1);
        // A characteristic the rank 5 does not offer deviates, and is raised.
        let other = chosen.with_raised_characteristic(Voie::Crime, Characteristic::For);
        assert_eq!(
            growth(&other),
            [Category::Deviation(co_kernel::Severity::Standard)]
        );
        assert_eq!(other.characteristics()[Characteristic::For].get(), 1);
        assert_eq!(
            other.without_raised_characteristic(Voie::Crime),
            ranks(Character::new(), Voie::Crime)
        );
    }

    #[test]
    fn growth_after_creation_never_breaks_the_creation_limits() {
        // A starting +5 raised by a rank 5 bought with XP.
        let character = (1..=4)
            .fold(
                Character::new().with_carac_points(Characteristic::Int, 5),
                |c, rank| c.with_rank(Voie::Langues, rank),
            )
            .with_experience(5)
            .with_xp_spending(crate::XpSpending::new().with_rank(Voie::Langues, 5));
        assert_eq!(character.characteristics()[Characteristic::Int].get(), 6);
        assert!(
            carac_violations(&character)
                .iter()
                .all(|v| v.rule() != &CARAC_LIMITS)
        );
    }
}
