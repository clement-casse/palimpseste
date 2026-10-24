//! Violations: computed findings about a character.

use std::borrow::Cow;

use crate::{AbilityId, Adjustment, Author, Characteristic, VoieId};

/// Stable identifier of a rule, such as `CO.VOIE.ORDER`.
///
/// Declared as a constant with [`RuleId::from_static`], or built from an
/// owned string with [`RuleId::new`] (a waiver loaded from storage). Both
/// compare by content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(Cow<'static, str>);

impl RuleId {
    /// Creates an identifier from an owned string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    /// Creates an identifier from a static string, usable in a constant.
    #[must_use]
    pub const fn from_static(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// Returns the identifier as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The mechanic that emitted a violation. Used for tracing, never for matching.
///
/// Mechanics are code and never persisted, so the name is static.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mechanic(&'static str);

impl Mechanic {
    /// Creates a mechanic name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Returns the name as a string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// An engine's name for one of its values: a derived value, a budget, or
/// another value such as the level.
///
/// Declared as a constant with [`ValueId::from_static`], or built from an
/// owned string with [`ValueId::new`] (a waiver loaded from storage). Both
/// compare by content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueId(Cow<'static, str>);

impl ValueId {
    /// Creates an identifier from an owned string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    /// Creates an identifier from a static string, usable in a constant.
    #[must_use]
    pub const fn from_static(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// Returns the identifier as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What a violation is about.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subject {
    /// The character as a whole.
    Character,
    /// One characteristic.
    Characteristic(Characteristic),
    /// One voie.
    Voie(VoieId),
    /// One ability.
    Ability(AbilityId),
    /// A derived value or a budget of the engine.
    Value(ValueId),
}

/// How strongly the book states a broken rule.
///
/// Display only: a waiver accepts a deviation whatever its severity, and
/// the Game Master can waive an `Absolute` rule like any other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// The book calls the rule indicative.
    Recommended,
    /// A book rule the Game Master may waive.
    Standard,
    /// The book says never.
    Absolute,
}

/// The kind of a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// A required choice is missing.
    Incomplete,
    /// A rule is broken; only this category can be waived.
    Deviation(Severity),
    /// A value cannot be computed from the current inputs.
    Uncomputable,
    /// Information only, such as a manual adjustment; never calls for action.
    Notice,
}

/// The value of a violation parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParameterValue {
    /// A number.
    Integer(i64),
    /// A text.
    Text(String),
}

impl From<i32> for ParameterValue {
    fn from(value: i32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i64> for ParameterValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<&str> for ParameterValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for ParameterValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

/// A named value describing a violation, for display only.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parameter {
    name: String,
    value: ParameterValue,
}

impl Parameter {
    /// Returns the parameter name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parameter value.
    #[must_use]
    pub const fn value(&self) -> &ParameterValue {
        &self.value
    }
}

/// A computed finding about a character.
///
/// The derived order (rule, mechanic, subject, category, parameters) is the
/// stable order of a violation report.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Violation {
    rule: RuleId,
    mechanic: Mechanic,
    subject: Subject,
    category: Category,
    parameters: Vec<Parameter>,
}

impl Violation {
    /// Creates a violation with no parameter.
    #[must_use]
    pub fn new(rule: RuleId, mechanic: Mechanic, subject: Subject, category: Category) -> Self {
        Self {
            rule,
            mechanic,
            subject,
            category,
            parameters: Vec::new(),
        }
    }

    /// Creates an [`Incomplete`](Category::Incomplete) violation.
    #[must_use]
    pub fn incomplete(rule: RuleId, mechanic: Mechanic, subject: Subject) -> Self {
        Self::new(rule, mechanic, subject, Category::Incomplete)
    }

    /// Creates a [`Deviation`](Category::Deviation) violation of `severity`.
    #[must_use]
    pub fn deviation(
        rule: RuleId,
        mechanic: Mechanic,
        subject: Subject,
        severity: Severity,
    ) -> Self {
        Self::new(rule, mechanic, subject, Category::Deviation(severity))
    }

    /// Creates an [`Uncomputable`](Category::Uncomputable) violation.
    #[must_use]
    pub fn uncomputable(rule: RuleId, mechanic: Mechanic, subject: Subject) -> Self {
        Self::new(rule, mechanic, subject, Category::Uncomputable)
    }

    /// Creates a [`Notice`](Category::Notice) violation.
    #[must_use]
    pub fn notice(rule: RuleId, mechanic: Mechanic, subject: Subject) -> Self {
        Self::new(rule, mechanic, subject, Category::Notice)
    }

    /// Adds a parameter.
    #[must_use]
    pub fn with_parameter(
        mut self,
        name: impl Into<String>,
        value: impl Into<ParameterValue>,
    ) -> Self {
        self.parameters.push(Parameter {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// Returns the rule broken or reported.
    #[must_use]
    pub const fn rule(&self) -> &RuleId {
        &self.rule
    }

    /// Returns the mechanic that emitted the violation.
    #[must_use]
    pub const fn mechanic(&self) -> Mechanic {
        self.mechanic
    }

    /// Returns what the violation is about.
    #[must_use]
    pub const fn subject(&self) -> &Subject {
        &self.subject
    }

    /// Returns the category.
    #[must_use]
    pub const fn category(&self) -> Category {
        self.category
    }

    /// Returns the parameters, in the order they were added.
    #[must_use]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
}

/// The rule behind every adjustment notice (`CO.GM.ADJUSTMENT`).
///
/// Lets a caller recognise adjustment notices without comparing strings.
pub const ADJUSTMENT_RULE: RuleId = RuleId::from_static("CO.GM.ADJUSTMENT");

/// The mechanic behind every adjustment notice.
const KERNEL: Mechanic = Mechanic::new("kernel");

/// Reports a manual adjustment applied to `subject` ([`ADJUSTMENT_RULE`]).
///
/// A project rule, not a book rule: every manual adjustment surfaces as a
/// `Notice`.
///
/// For a derived value, prefer [`DerivedValue::adjustment_notices`], which
/// reads the adjustments from the breakdown.
///
/// [`DerivedValue::adjustment_notices`]: crate::DerivedValue::adjustment_notices
#[must_use]
pub fn adjustment_notice(subject: Subject, adjustment: &Adjustment) -> Violation {
    notice_for(
        subject,
        adjustment.amount(),
        adjustment.author(),
        adjustment.reason(),
    )
}

/// The notice for an adjustment of `amount` by `author` on `subject`.
pub(crate) fn notice_for(
    subject: Subject,
    amount: i32,
    author: &Author,
    reason: &str,
) -> Violation {
    Violation::notice(ADJUSTMENT_RULE, KERNEL, subject)
        .with_parameter("amount", amount)
        .with_parameter("author", author.as_str())
        .with_parameter("reason", reason)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn static_and_owned_ids_compare_by_content() {
        const RULE: RuleId = RuleId::from_static("TEST.RULE");
        const VALUE: ValueId = ValueId::from_static("test-value");
        let owned_rule = RuleId::new("TEST.RULE".to_owned());
        assert_eq!(RuleId::from_static("TEST.RULE"), owned_rule);
        let owned_value = ValueId::new("test-value".to_owned());
        assert_eq!(RULE, owned_rule);
        assert_eq!(RULE.cmp(&owned_rule), std::cmp::Ordering::Equal);
        assert_eq!(HashSet::from([RULE, owned_rule]).len(), 1);
        assert_eq!(VALUE, owned_value);
        assert_eq!(VALUE.cmp(&owned_value), std::cmp::Ordering::Equal);
        assert_eq!(HashSet::from([VALUE, owned_value]).len(), 1);
    }

    fn violation(category: Category) -> Violation {
        Violation::new(
            RuleId::new("TEST.RULE"),
            Mechanic::new("test-mechanic"),
            Subject::Character,
            category,
        )
    }

    #[test]
    fn every_category_can_be_built() {
        for category in [
            Category::Incomplete,
            Category::Deviation(Severity::Standard),
            Category::Uncomputable,
            Category::Notice,
        ] {
            let violation = violation(category);
            assert_eq!(violation.category(), category);
            assert_eq!(violation.rule().as_str(), "TEST.RULE");
            assert_eq!(violation.mechanic().as_str(), "test-mechanic");
            assert_eq!(violation.subject(), &Subject::Character);
        }
    }

    #[test]
    fn category_constructors_set_their_category() {
        const RULE: RuleId = RuleId::from_static("TEST.RULE");
        const MECHANIC: Mechanic = Mechanic::new("test-mechanic");
        let built = [
            Violation::incomplete(RULE, MECHANIC, Subject::Character),
            Violation::deviation(RULE, MECHANIC, Subject::Character, Severity::Standard),
            Violation::uncomputable(RULE, MECHANIC, Subject::Character),
            Violation::notice(RULE, MECHANIC, Subject::Character),
        ];
        let expected = [
            Category::Incomplete,
            Category::Deviation(Severity::Standard),
            Category::Uncomputable,
            Category::Notice,
        ];
        for (violation, category) in built.into_iter().zip(expected) {
            assert_eq!(violation, self::violation(category));
        }
    }

    #[test]
    fn deviation_keeps_its_severity() {
        const RULE: RuleId = RuleId::from_static("TEST.RULE");
        const MECHANIC: Mechanic = Mechanic::new("test-mechanic");
        for severity in [
            Severity::Recommended,
            Severity::Standard,
            Severity::Absolute,
        ] {
            let deviation = Violation::deviation(RULE, MECHANIC, Subject::Character, severity);
            assert_eq!(deviation.category(), Category::Deviation(severity));
        }
    }

    #[test]
    fn parameter_values_convert_from_numbers_and_text() {
        assert_eq!(ParameterValue::from(-3_i32), ParameterValue::Integer(-3));
        assert_eq!(
            ParameterValue::from(i64::MAX),
            ParameterValue::Integer(i64::MAX)
        );
        assert_eq!(
            ParameterValue::from("text"),
            ParameterValue::Text("text".to_owned())
        );
        assert_eq!(
            ParameterValue::from("owned".to_owned()),
            ParameterValue::Text("owned".to_owned())
        );
        let violation = violation(Category::Deviation(Severity::Standard))
            .with_parameter("value", 7)
            .with_parameter("label", "test");
        assert_eq!(
            violation.parameters()[0].value(),
            &ParameterValue::Integer(7)
        );
        assert_eq!(
            violation.parameters()[1].value(),
            &ParameterValue::Text("test".to_owned())
        );
    }

    #[test]
    fn parameters_keep_insertion_order() {
        let violation = violation(Category::Deviation(Severity::Standard))
            .with_parameter("value", ParameterValue::Integer(7))
            .with_parameter("max", ParameterValue::Integer(5));
        let names: Vec<_> = violation.parameters().iter().map(Parameter::name).collect();
        assert_eq!(names, ["value", "max"]);
        assert_eq!(
            violation.parameters()[0].value(),
            &ParameterValue::Integer(7)
        );
    }

    #[test]
    fn adjustment_notice_describes_the_adjustment() {
        let subject = Subject::Value(ValueId::new("test-value"));
        let adjustment = Adjustment::new(-2, Author::new("gm"), "test reason");
        let notice = adjustment_notice(subject.clone(), &adjustment);
        assert_eq!(notice.rule(), &ADJUSTMENT_RULE);
        assert_eq!(notice.rule().as_str(), "CO.GM.ADJUSTMENT");
        assert_eq!(notice.mechanic().as_str(), "kernel");
        assert_eq!(notice.subject(), &subject);
        assert_eq!(notice.category(), Category::Notice);
        let parameters: Vec<_> = notice
            .parameters()
            .iter()
            .map(|p| (p.name(), p.value()))
            .collect();
        assert_eq!(
            parameters,
            [
                ("amount", &ParameterValue::Integer(-2)),
                ("author", &ParameterValue::Text("gm".to_owned())),
                ("reason", &ParameterValue::Text("test reason".to_owned())),
            ]
        );
    }
}
