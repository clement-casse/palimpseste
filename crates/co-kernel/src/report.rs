//! Violation report: violations matched against waivers.

use crate::{Category, Violation, Waiver};

/// A violation and the waiver accepting it, if any.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReportEntry {
    violation: Violation,
    waiver: Option<Waiver>,
}

impl ReportEntry {
    /// Returns the violation.
    #[must_use]
    pub const fn violation(&self) -> &Violation {
        &self.violation
    }

    /// Returns the waiver accepting the violation, if any.
    #[must_use]
    pub const fn waiver(&self) -> Option<&Waiver> {
        self.waiver.as_ref()
    }

    /// Whether a waiver accepts the violation.
    #[must_use]
    pub const fn is_waived(&self) -> bool {
        self.waiver.is_some()
    }
}

/// Every violation of a character, in a stable order, matched against waivers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViolationReport {
    entries: Vec<ReportEntry>,
    unused_waivers: Vec<Waiver>,
}

impl ViolationReport {
    /// Builds the report.
    ///
    /// Violations are sorted, so the report does not depend on the order in
    /// which mechanics emitted them. Each `Deviation` takes the first waiver
    /// that matches it. Waivers that accepted no violation are kept, in input
    /// order, as unused: this includes a duplicate of an earlier waiver.
    #[must_use]
    pub fn new(mut violations: Vec<Violation>, waivers: &[Waiver]) -> Self {
        violations.sort();
        let mut used = vec![false; waivers.len()];
        let entries = violations
            .into_iter()
            .map(|violation| {
                let waiver = waivers
                    .iter()
                    .zip(used.iter_mut())
                    .find(|(waiver, _)| waiver.matches(&violation))
                    .map(|(waiver, used)| {
                        *used = true;
                        waiver.clone()
                    });
                ReportEntry { violation, waiver }
            })
            .collect();
        let unused_waivers = waivers
            .iter()
            .zip(&used)
            .filter(|(_, used)| !**used)
            .map(|(waiver, _)| waiver.clone())
            .collect();
        Self {
            entries,
            unused_waivers,
        }
    }

    /// Returns every entry, waived or not.
    #[must_use]
    pub fn entries(&self) -> &[ReportEntry] {
        &self.entries
    }

    /// Iterates over the entries that call for action: violations no waiver
    /// accepts, notices excluded.
    pub fn open(&self) -> impl Iterator<Item = &ReportEntry> {
        self.entries
            .iter()
            .filter(|entry| !entry.is_waived() && entry.violation.category() != Category::Notice)
    }

    /// Iterates over the notices, which are information only.
    pub fn notices(&self) -> impl Iterator<Item = &ReportEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.violation.category() == Category::Notice)
    }

    /// Returns the waivers that matched no violation.
    #[must_use]
    pub fn unused_waivers(&self) -> &[Waiver] {
        &self.unused_waivers
    }

    /// Whether nothing calls for action: every violation is waived or is a
    /// notice.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.open().next().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Adjustment, Author, Mechanic, RuleId, Subject, VoieId, adjustment_notice};

    fn violation(rule: &str, category: Category) -> Violation {
        Violation::new(
            RuleId::new(rule),
            Mechanic::new("test-mechanic"),
            Subject::Voie(VoieId::new("test-voie")),
            category,
        )
    }

    fn waiver(rule: &str) -> Waiver {
        Waiver::new(
            RuleId::new(rule),
            Subject::Voie(VoieId::new("test-voie")),
            Author::new("gm"),
            "test reason",
        )
    }

    #[test]
    fn report_does_not_depend_on_input_order() {
        let a = violation("TEST.A", Category::Incomplete);
        let b = violation("TEST.B", Category::Uncomputable);
        let c = violation("TEST.C", Category::Deviation);
        let waivers = [waiver("TEST.C")];
        let forward = ViolationReport::new(vec![a.clone(), b.clone(), c.clone()], &waivers);
        let backward = ViolationReport::new(vec![c, b, a], &waivers);
        assert_eq!(forward, backward);
        let rules: Vec<_> = forward
            .entries()
            .iter()
            .map(|e| e.violation().rule().as_str())
            .collect();
        assert_eq!(rules, ["TEST.A", "TEST.B", "TEST.C"]);
    }

    #[test]
    fn waived_deviation_is_not_open() {
        let report = ViolationReport::new(
            vec![violation("TEST.DEVIATION", Category::Deviation)],
            &[waiver("TEST.DEVIATION")],
        );
        assert!(report.entries()[0].is_waived());
        assert_eq!(
            report.entries()[0].waiver(),
            Some(&waiver("TEST.DEVIATION"))
        );
        assert_eq!(report.open().count(), 0);
        assert!(report.is_clean());
        assert_eq!(report.unused_waivers(), []);
    }

    #[test]
    fn unwaived_findings_other_than_notices_are_open() {
        for category in [
            Category::Incomplete,
            Category::Deviation,
            Category::Uncomputable,
        ] {
            let report = ViolationReport::new(vec![violation("TEST.RULE", category)], &[]);
            let open: Vec<_> = report.open().map(ReportEntry::violation).collect();
            assert_eq!(open, [&violation("TEST.RULE", category)]);
            assert!(!report.is_clean());
        }
    }

    #[test]
    fn deviation_waived_for_another_rule_stays_open() {
        let report = ViolationReport::new(
            vec![violation("TEST.DEVIATION", Category::Deviation)],
            &[waiver("TEST.OTHER")],
        );
        assert!(!report.entries()[0].is_waived());
        assert_eq!(report.open().count(), 1);
        assert!(!report.is_clean());
    }

    #[test]
    fn one_waiver_accepts_deviations_whatever_their_parameters() {
        let low = violation("TEST.DEVIATION", Category::Deviation).with_parameter("value", 6);
        let high = violation("TEST.DEVIATION", Category::Deviation).with_parameter("value", 9);
        let report = ViolationReport::new(vec![low, high], &[waiver("TEST.DEVIATION")]);
        assert!(report.entries().iter().all(ReportEntry::is_waived));
        assert!(report.is_clean());
        assert_eq!(report.unused_waivers(), []);
    }

    #[test]
    fn duplicate_waiver_is_unused() {
        let first = waiver("TEST.DEVIATION");
        let duplicate = Waiver::new(
            RuleId::new("TEST.DEVIATION"),
            Subject::Voie(VoieId::new("test-voie")),
            Author::new("player"),
            "duplicate reason",
        );
        let report = ViolationReport::new(
            vec![violation("TEST.DEVIATION", Category::Deviation)],
            &[first.clone(), duplicate.clone()],
        );
        assert_eq!(report.entries()[0].waiver(), Some(&first));
        assert_eq!(report.unused_waivers(), [duplicate]);
    }

    #[test]
    fn notice_cannot_be_waived() {
        let report = ViolationReport::new(
            vec![violation("TEST.NOTICE", Category::Notice)],
            &[waiver("TEST.NOTICE")],
        );
        assert!(!report.entries()[0].is_waived());
        assert_eq!(report.unused_waivers(), [waiver("TEST.NOTICE")]);
        assert_eq!(report.open().count(), 0);
        assert_eq!(report.notices().count(), 1);
        assert!(report.is_clean());
    }

    #[test]
    fn notices_are_listed_apart_from_open_entries() {
        let adjustment = Adjustment::new(1, Author::new("gm"), "test reason");
        let notice = adjustment_notice(Subject::Voie(VoieId::new("test-voie")), &adjustment);
        let incomplete = violation("TEST.INCOMPLETE", Category::Incomplete);
        let report = ViolationReport::new(vec![notice.clone(), incomplete.clone()], &[]);
        let open: Vec<_> = report.open().map(ReportEntry::violation).collect();
        let notices: Vec<_> = report.notices().map(ReportEntry::violation).collect();
        assert_eq!(open, [&incomplete]);
        assert_eq!(notices, [&notice]);
        assert!(!report.is_clean());
    }

    #[test]
    fn adjusted_character_can_be_clean() {
        let adjustment = Adjustment::new(-1, Author::new("gm"), "test reason");
        let notice = adjustment_notice(Subject::Character, &adjustment);
        let report = ViolationReport::new(vec![notice], &[]);
        assert_eq!(report.notices().count(), 1);
        assert!(report.is_clean());
    }

    #[test]
    fn waiver_matching_nothing_is_unused() {
        let report = ViolationReport::new(
            vec![violation("TEST.DEVIATION", Category::Deviation)],
            &[waiver("TEST.STALE"), waiver("TEST.DEVIATION")],
        );
        assert!(report.is_clean());
        assert_eq!(report.unused_waivers(), [waiver("TEST.STALE")]);
    }

    #[test]
    fn empty_report_is_clean() {
        let report = ViolationReport::new(Vec::new(), &[]);
        assert_eq!(report.entries(), []);
        assert!(report.is_clean());
    }

    #[test]
    fn identical_violations_are_both_kept() {
        let incomplete = violation("TEST.INCOMPLETE", Category::Incomplete);
        let report = ViolationReport::new(vec![incomplete.clone(), incomplete], &[]);
        assert_eq!(report.entries().len(), 2);
        assert_eq!(report.open().count(), 2);
    }
}
