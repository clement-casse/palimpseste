//! Waivers: attributed acceptance of a deviation.

use crate::{Author, Category, RuleId, Subject, Violation};

/// The acceptance of a `Deviation` from `rule` on `subject`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Waiver {
    rule: RuleId,
    subject: Subject,
    author: Author,
    reason: String,
}

impl Waiver {
    /// Creates a waiver.
    #[must_use]
    pub fn new(rule: RuleId, subject: Subject, author: Author, reason: impl Into<String>) -> Self {
        Self {
            rule,
            subject,
            author,
            reason: reason.into(),
        }
    }

    /// Returns the waived rule.
    #[must_use]
    pub const fn rule(&self) -> &RuleId {
        &self.rule
    }

    /// Returns the subject the waiver applies to.
    #[must_use]
    pub const fn subject(&self) -> &Subject {
        &self.subject
    }

    /// Returns who accepted the deviation.
    #[must_use]
    pub const fn author(&self) -> &Author {
        &self.author
    }

    /// Returns why the deviation was accepted.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Whether this waiver accepts `violation`: same rule, same subject, and
    /// a `Deviation` of any severity. Parameters are ignored.
    pub(crate) fn matches(&self, violation: &Violation) -> bool {
        matches!(violation.category(), Category::Deviation(_))
            && violation.rule() == &self.rule
            && violation.subject() == &self.subject
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Characteristic, Mechanic, ParameterValue, Severity};

    fn waiver() -> Waiver {
        Waiver::new(
            RuleId::new("TEST.RULE"),
            Subject::Characteristic(Characteristic::For),
            Author::new("gm"),
            "test reason",
        )
    }

    fn violation(rule: &str, subject: Subject, category: Category) -> Violation {
        Violation::new(
            RuleId::new(rule),
            Mechanic::new("test-mechanic"),
            subject,
            category,
        )
    }

    #[test]
    fn matches_a_deviation_with_same_rule_and_subject() {
        let deviation = violation(
            "TEST.RULE",
            Subject::Characteristic(Characteristic::For),
            Category::Deviation(Severity::Standard),
        )
        .with_parameter("value", ParameterValue::Integer(9));
        assert!(waiver().matches(&deviation));
    }

    #[test]
    fn matches_a_deviation_whatever_its_severity() {
        for severity in [
            Severity::Recommended,
            Severity::Standard,
            Severity::Absolute,
        ] {
            let deviation = violation(
                "TEST.RULE",
                Subject::Characteristic(Characteristic::For),
                Category::Deviation(severity),
            );
            assert!(waiver().matches(&deviation));
        }
    }

    #[test]
    fn owned_rule_id_matches_a_constant_one() {
        const RULE: RuleId = RuleId::from_static("TEST.RULE");
        let deviation = Violation::new(
            RULE,
            Mechanic::new("test-mechanic"),
            Subject::Characteristic(Characteristic::For),
            Category::Deviation(Severity::Standard),
        );
        assert!(waiver().matches(&deviation));
    }

    #[test]
    fn does_not_match_another_rule_or_subject() {
        let for_subject = Subject::Characteristic(Characteristic::For);
        let other_rule = violation(
            "TEST.OTHER",
            for_subject,
            Category::Deviation(Severity::Standard),
        );
        let other_subject = violation(
            "TEST.RULE",
            Subject::Characteristic(Characteristic::Agi),
            Category::Deviation(Severity::Standard),
        );
        assert!(!waiver().matches(&other_rule));
        assert!(!waiver().matches(&other_subject));
    }

    #[test]
    fn does_not_match_other_categories() {
        for category in [
            Category::Incomplete,
            Category::Uncomputable,
            Category::Notice,
        ] {
            let finding = violation(
                "TEST.RULE",
                Subject::Characteristic(Characteristic::For),
                category,
            );
            assert!(!waiver().matches(&finding));
        }
    }
}
