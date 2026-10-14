//! Derived values together with the contributions that produced them.

use crate::{
    AbilityId, Adjustment, Author, Characteristic, Characteristics, RuleId, Subject, ValueId,
    Violation, violation::notice_for,
};

/// Where a contribution to a derived value comes from.
///
/// Engines cannot add variants, so `Rule` and `Value` stay engine-agnostic:
/// together they let every term of a formula appear in the breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    /// A characteristic of the character.
    Characteristic(Characteristic),
    /// An ability of the character.
    Ability(AbilityId),
    /// A term the rule's formula sets itself, such as a constant.
    Rule(RuleId),
    /// A term taken from another value the engine names, such as the level or
    /// an armour bonus.
    Value(ValueId),
    /// A manual adjustment.
    Adjustment {
        /// Who made the adjustment.
        author: Author,
        /// Why the adjustment was made.
        reason: String,
    },
}

/// One term of a derived value: a source and its signed amount.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contribution {
    source: Source,
    amount: i32,
}

impl Contribution {
    /// Creates a contribution of `amount` from `source`.
    #[must_use]
    pub const fn new(source: Source, amount: i32) -> Self {
        Self { source, amount }
    }

    /// Returns where the contribution comes from.
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }

    /// Returns the signed amount.
    #[must_use]
    pub const fn amount(&self) -> i32 {
        self.amount
    }
}

/// A computed value and its breakdown.
///
/// The result of a computation, never stored on a character.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DerivedValue {
    contributions: Vec<Contribution>,
}

impl DerivedValue {
    /// Creates a value with no contribution.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a contribution of `amount` from `source`.
    #[must_use]
    pub fn with(mut self, source: Source, amount: i32) -> Self {
        self.contributions.push(Contribution::new(source, amount));
        self
    }

    /// Adds the value of `characteristic` taken from `characteristics`.
    #[must_use]
    pub fn with_characteristic(
        self,
        characteristics: &Characteristics,
        characteristic: Characteristic,
    ) -> Self {
        self.with(
            Source::Characteristic(characteristic),
            characteristics[characteristic].into(),
        )
    }

    /// Adds `adjustment` as a contribution.
    #[must_use]
    pub fn with_adjustment(self, adjustment: &Adjustment) -> Self {
        let source = Source::Adjustment {
            author: adjustment.author().clone(),
            reason: adjustment.reason().to_owned(),
        };
        self.with(source, adjustment.amount())
    }

    /// Returns the contributions, in the order they were added.
    #[must_use]
    pub fn contributions(&self) -> &[Contribution] {
        &self.contributions
    }

    /// Returns the sum of the contributions, clamped to the `i32` range.
    ///
    /// The sum is computed exactly before clamping, so the result does not
    /// depend on the order of the contributions.
    #[must_use]
    pub fn total(&self) -> i32 {
        let sum = self
            .contributions
            .iter()
            .fold(0_i64, |sum, c| sum.saturating_add(i64::from(c.amount)));
        i32::try_from(sum).unwrap_or(if sum < 0 { i32::MIN } else { i32::MAX })
    }

    /// Yields one adjustment notice on `subject` for each adjustment in the
    /// breakdown, in breakdown order.
    ///
    /// The notices are read from the breakdown, so they always match the
    /// adjustments it contains.
    pub fn adjustment_notices<'a>(
        &'a self,
        subject: &'a Subject,
    ) -> impl Iterator<Item = Violation> + 'a {
        self.contributions
            .iter()
            .filter_map(move |contribution| match &contribution.source {
                Source::Adjustment { author, reason } => Some(notice_for(
                    subject.clone(),
                    contribution.amount,
                    author,
                    reason,
                )),
                _ => None,
            })
    }
}

impl FromIterator<Contribution> for DerivedValue {
    fn from_iter<I: IntoIterator<Item = Contribution>>(iter: I) -> Self {
        Self {
            contributions: iter.into_iter().collect(),
        }
    }
}

impl Extend<Contribution> for DerivedValue {
    fn extend<I: IntoIterator<Item = Contribution>>(&mut self, iter: I) {
        self.contributions.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_value_totals_zero() {
        let value = DerivedValue::new();
        assert_eq!(value.total(), 0);
        assert_eq!(value.contributions(), []);
    }

    #[test]
    fn breakdown_keeps_insertion_order_and_sums() {
        let value = DerivedValue::new()
            .with(Source::Characteristic(Characteristic::Per), 2)
            .with(Source::Ability(AbilityId::new("test-ability")), -3)
            .with(Source::Characteristic(Characteristic::Agi), 4);
        let sources: Vec<_> = value
            .contributions()
            .iter()
            .map(Contribution::source)
            .collect();
        assert_eq!(
            sources,
            [
                &Source::Characteristic(Characteristic::Per),
                &Source::Ability(AbilityId::new("test-ability")),
                &Source::Characteristic(Characteristic::Agi),
            ]
        );
        assert_eq!(value.total(), 3);
    }

    #[test]
    fn rule_and_value_terms_appear_in_breakdown_and_total() {
        const RULE: RuleId = RuleId::from_static("TEST.RULE");
        const VALUE: ValueId = ValueId::from_static("test-value");
        let value = DerivedValue::new()
            .with(Source::Rule(RULE), 10)
            .with(Source::Characteristic(Characteristic::Agi), 2)
            .with(Source::Value(VALUE), 3);
        let sources: Vec<_> = value
            .contributions()
            .iter()
            .map(Contribution::source)
            .collect();
        assert_eq!(
            sources,
            [
                &Source::Rule(RULE),
                &Source::Characteristic(Characteristic::Agi),
                &Source::Value(VALUE),
            ]
        );
        assert_eq!(value.total(), 15);
    }

    #[test]
    fn total_saturates() {
        let source = Source::Characteristic(Characteristic::Con);
        let high = DerivedValue::new()
            .with(source.clone(), i32::MAX)
            .with(source.clone(), 1);
        assert_eq!(high.total(), i32::MAX);
        let low = DerivedValue::new()
            .with(source.clone(), i32::MIN)
            .with(source, -1);
        assert_eq!(low.total(), i32::MIN);
    }

    #[test]
    fn total_clamps_the_exact_sum_whatever_the_order() {
        let source = Source::Characteristic(Characteristic::Con);
        let value: DerivedValue = [i32::MAX, 1, -1]
            .into_iter()
            .map(|amount| Contribution::new(source.clone(), amount))
            .collect();
        assert_eq!(value.total(), i32::MAX);
        let value: DerivedValue = [i32::MIN, -1, 1]
            .into_iter()
            .map(|amount| Contribution::new(source.clone(), amount))
            .collect();
        assert_eq!(value.total(), i32::MIN);
    }

    #[test]
    fn adjustment_appears_in_breakdown_and_total() {
        let adjustment = Adjustment::new(-1, Author::new("gm"), "test reason");
        let value = DerivedValue::new()
            .with(Source::Characteristic(Characteristic::Vol), 2)
            .with_adjustment(&adjustment);
        let last = &value.contributions()[1];
        assert_eq!(
            last.source(),
            &Source::Adjustment {
                author: Author::new("gm"),
                reason: "test reason".to_owned(),
            }
        );
        assert_eq!(last.amount(), -1);
        assert_eq!(value.total(), 1);
    }

    #[test]
    fn adjustment_notices_follow_the_breakdown() {
        let subject = Subject::Value(ValueId::from_static("test-value"));
        let first = Adjustment::new(2, Author::new("gm"), "first reason");
        let second = Adjustment::new(-1, Author::new("player"), "second reason");
        let value = DerivedValue::new()
            .with_adjustment(&first)
            .with(Source::Characteristic(Characteristic::Int), 1)
            .with_adjustment(&second);
        let notices: Vec<_> = value.adjustment_notices(&subject).collect();
        assert_eq!(
            notices,
            [
                crate::adjustment_notice(subject.clone(), &first),
                crate::adjustment_notice(subject.clone(), &second),
            ]
        );
    }

    #[test]
    fn no_adjustment_means_no_notice() {
        let value = DerivedValue::new().with(Source::Characteristic(Characteristic::Int), 1);
        assert_eq!(value.adjustment_notices(&Subject::Character).count(), 0);
    }

    #[test]
    fn collect_and_extend_keep_order() {
        let mut value: DerivedValue = [
            Contribution::new(Source::Characteristic(Characteristic::For), 2),
            Contribution::new(Source::Ability(AbilityId::new("test-ability")), 1),
        ]
        .into_iter()
        .collect();
        value.extend([Contribution::new(
            Source::Value(ValueId::from_static("test-value")),
            -4,
        )]);
        let amounts: Vec<_> = value
            .contributions()
            .iter()
            .map(Contribution::amount)
            .collect();
        assert_eq!(amounts, [2, 1, -4]);
        assert_eq!(value.total(), -1);
    }

    #[test]
    fn with_characteristic_reads_the_matching_value() {
        let characteristics = Characteristics::from_fn(|c| match c {
            Characteristic::Per => 3.into(),
            _ => (-1).into(),
        });
        let value = DerivedValue::new().with_characteristic(&characteristics, Characteristic::Per);
        assert_eq!(
            value.contributions(),
            [Contribution::new(
                Source::Characteristic(Characteristic::Per),
                3
            )]
        );
    }
}
