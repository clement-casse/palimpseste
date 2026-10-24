//! Age and age bracket (COC.CRE.AGE-BRACKET).

use co_kernel::{DerivedValue, Severity, Source, Subject, Violation};

use crate::{
    Character, Trait, Value,
    rules::{AGE, AGE_BRACKET, AGE_MECHANIC},
};

/// The age of a character, in years.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Age(u16);

impl Age {
    /// Creates an age.
    #[must_use]
    pub const fn new(years: u16) -> Self {
        Self(years)
    }

    /// Returns the age in years.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl From<u16> for Age {
    fn from(years: u16) -> Self {
        Self(years)
    }
}

/// The age bracket, which sets the creation budgets and limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AgeBracket {
    /// 10 to 15 years, with the trait D'un autre âge (D-16, D-18).
    Adolescent,
    /// 16 to 30 years.
    From16To30,
    /// 31 to 50 years.
    From31To50,
    /// 51 to 65 years.
    From51To65,
    /// 66 years and more, with the trait D'un autre âge (D-16, D-18).
    Elderly,
}

impl AgeBracket {
    /// Returns the bracket of `age`, or `None` below 10 years.
    #[must_use]
    pub const fn of(age: Age) -> Option<Self> {
        match age.0 {
            0..=9 => None,
            10..=15 => Some(Self::Adolescent),
            16..=30 => Some(Self::From16To30),
            31..=50 => Some(Self::From31To50),
            51..=65 => Some(Self::From51To65),
            _ => Some(Self::Elderly),
        }
    }

    /// The French display name of the bracket.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Adolescent => "10-15 ans (adolescent)",
            Self::From16To30 => "16-30 ans",
            Self::From31To50 => "31-50 ans",
            Self::From51To65 => "51-65 ans",
            Self::Elderly => "66 ans et plus (âgé)",
        }
    }

    /// Whether the bracket lies outside 16-65 and calls for D'un autre âge.
    const fn is_out_of_age(self) -> bool {
        matches!(self, Self::Adolescent | Self::Elderly)
    }

    /// Returns the creation limits of the bracket, before the changes
    /// D'un autre âge makes to the budgets (D-16, D-18).
    pub(crate) const fn limits(self) -> Limits {
        match self {
            Self::Adolescent | Self::From16To30 => Limits {
                capacity_points: 4,
                max_voies: 4,
                max_rank: 2,
                carac_points: 10,
                physical_max: 5,
                physical_min: 0,
            },
            Self::From31To50 => Limits {
                capacity_points: 7,
                max_voies: 5,
                max_rank: 3,
                carac_points: 8,
                physical_max: 4,
                physical_min: -1,
            },
            Self::From51To65 | Self::Elderly => Limits {
                capacity_points: 10,
                max_voies: 6,
                max_rank: 4,
                carac_points: 6,
                physical_max: 3,
                physical_min: -2,
            },
        }
    }
}

/// The creation limits of an age bracket (data, COC.CRE.AGE-BRACKET).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Limits {
    pub(crate) capacity_points: i32,
    pub(crate) max_voies: i32,
    pub(crate) max_rank: i32,
    pub(crate) carac_points: i32,
    pub(crate) physical_max: i32,
    pub(crate) physical_min: i32,
}

/// Returns the creation budget `value` from the age bracket, with the change
/// D'un autre âge makes to it; `None` without a bracket.
pub(crate) fn budget(character: &Character, value: Value) -> Option<DerivedValue> {
    let bracket = character.age_bracket()?;
    let limits = bracket.limits();
    let (base, adolescent, elderly) = match value {
        Value::CaracPoints => (limits.carac_points, -2, 0),
        Value::CapacityPoints => (limits.capacity_points, -1, 2),
        Value::MaxVoies => (limits.max_voies, 0, 0),
        Value::MaxRank => (limits.max_rank, 0, 0),
        // Unreachable: `Character::value` asks for the budgets only.
        _ => return None,
    };
    let budget = DerivedValue::new().with(Source::Rule(AGE_BRACKET), base);
    let change = match bracket {
        AgeBracket::Adolescent => adolescent,
        AgeBracket::Elderly => elderly,
        _ => 0,
    };
    Some(if change != 0 && character.has_trait(Trait::DUnAutreAge) {
        budget.with(Source::Value(Trait::DUnAutreAge.id()), change)
    } else {
        budget
    })
}

/// Checks the age against the bracket and the trait D'un autre âge.
pub(crate) fn check(character: &Character, violations: &mut Vec<Violation>) {
    let subject = || Subject::Value(AGE);
    let Some(age) = character.age() else {
        violations.push(Violation::incomplete(AGE_BRACKET, AGE_MECHANIC, subject()));
        return;
    };
    let Some(bracket) = AgeBracket::of(age) else {
        violations.push(
            Violation::uncomputable(AGE_BRACKET, AGE_MECHANIC, subject())
                .with_parameter("age", i32::from(age.get())),
        );
        return;
    };
    // Opposite cases get distinct subjects, so a waiver for one does not
    // carry over to the other.
    let has_trait = character.has_trait(Trait::DUnAutreAge);
    let subject = match (bracket.is_out_of_age(), has_trait) {
        (true, false) => subject(),
        (false, true) => Subject::Value(Trait::DUnAutreAge.id()),
        _ => return,
    };
    violations.push(
        Violation::deviation(AGE_BRACKET, AGE_MECHANIC, subject, Severity::Standard)
            .with_parameter("age", i32::from(age.get())),
    );
}

#[cfg(test)]
mod tests {
    use co_kernel::Category;

    use super::*;

    fn age_violations(character: &Character) -> Vec<Violation> {
        let mut violations = Vec::new();
        check(character, &mut violations);
        violations
    }

    fn budgets(character: &Character) -> [Option<i32>; 4] {
        [
            Value::CaracPoints,
            Value::CapacityPoints,
            Value::MaxVoies,
            Value::MaxRank,
        ]
        .map(|value| budget(character, value).map(|v| v.total()))
    }

    #[test]
    fn budgets_follow_the_bracket_table() {
        let budgets_at = |years| budgets(&Character::new().with_age(years));
        assert_eq!(budgets_at(20), [Some(10), Some(4), Some(4), Some(2)]);
        assert_eq!(budgets_at(40), [Some(8), Some(7), Some(5), Some(3)]);
        assert_eq!(budgets_at(60), [Some(6), Some(10), Some(6), Some(4)]);
        assert_eq!(budgets_at(8), [None; 4]);
    }

    #[test]
    fn d_un_autre_age_changes_the_budgets() {
        let adolescent = Character::new()
            .with_age(13)
            .with_trait(Trait::DUnAutreAge, 2);
        assert_eq!(budgets(&adolescent), [Some(8), Some(3), Some(4), Some(2)]);
        let elderly = adolescent.clone().with_age(70);
        assert_eq!(budgets(&elderly), [Some(6), Some(12), Some(6), Some(4)]);
        let source =
            budget(&elderly, Value::CapacityPoints).map(|v| v.contributions()[1].source().clone());
        assert_eq!(source, Some(Source::Value(Trait::DUnAutreAge.id())));
        // Without the trait, the closest bracket applies unchanged.
        let without_trait = Character::new().with_age(13);
        assert_eq!(
            budgets(&without_trait),
            [Some(10), Some(4), Some(4), Some(2)]
        );
    }

    #[test]
    fn brackets_follow_the_age() {
        let bracket = |years| AgeBracket::of(Age::new(years));
        assert_eq!(bracket(9), None);
        assert_eq!(bracket(10), Some(AgeBracket::Adolescent));
        assert_eq!(bracket(15), Some(AgeBracket::Adolescent));
        assert_eq!(bracket(16), Some(AgeBracket::From16To30));
        assert_eq!(bracket(30), Some(AgeBracket::From16To30));
        assert_eq!(bracket(31), Some(AgeBracket::From31To50));
        assert_eq!(bracket(50), Some(AgeBracket::From31To50));
        assert_eq!(bracket(51), Some(AgeBracket::From51To65));
        assert_eq!(bracket(65), Some(AgeBracket::From51To65));
        assert_eq!(bracket(66), Some(AgeBracket::Elderly));
    }

    #[test]
    fn missing_age_is_incomplete() {
        let violations = age_violations(&Character::new());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].category(), Category::Incomplete);
        assert_eq!(violations[0].rule(), &AGE_BRACKET);
    }

    #[test]
    fn age_below_every_bracket_is_uncomputable() {
        let violations = age_violations(&Character::new().with_age(8));
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].category(), Category::Uncomputable);
    }

    #[test]
    fn age_within_the_brackets_is_clean() {
        assert_eq!(age_violations(&Character::new().with_age(32)), []);
    }

    #[test]
    fn adolescent_needs_d_un_autre_age() {
        let adolescent = Character::new().with_age(12);
        let violations = age_violations(&adolescent);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].category(),
            Category::Deviation(Severity::Standard)
        );
        let with_trait = adolescent.with_trait(Trait::DUnAutreAge, 2);
        assert_eq!(age_violations(&with_trait), []);
    }

    #[test]
    fn d_un_autre_age_needs_an_adolescent_or_elderly_character() {
        let adult = Character::new()
            .with_age(40)
            .with_trait(Trait::DUnAutreAge, 2);
        let violations = age_violations(&adult);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].category(),
            Category::Deviation(Severity::Standard)
        );
        assert_eq!(
            violations[0].subject(),
            &Subject::Value(Trait::DUnAutreAge.id())
        );
        let elderly = adult.with_age(70);
        assert_eq!(age_violations(&elderly), []);
    }

    #[test]
    fn every_bracket_is_named() {
        let names: Vec<_> = [12, 20, 40, 60, 70]
            .into_iter()
            .filter_map(|years: u16| AgeBracket::of(years.into()))
            .map(AgeBracket::name)
            .collect();
        assert_eq!(
            names,
            [
                "10-15 ans (adolescent)",
                "16-30 ans",
                "31-50 ans",
                "51-65 ans",
                "66 ans et plus (âgé)",
            ]
        );
    }

    #[test]
    fn only_the_budgets_come_from_the_bracket() {
        let character = Character::new().with_age(20);
        assert_eq!(budget(&character, Value::Initiative), None);
        assert!(budget(&character, Value::MaxRank).is_some());
    }
}
