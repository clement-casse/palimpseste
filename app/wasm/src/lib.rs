//! The engines, compiled to WebAssembly for the front end.
//!
//! Each binding is a stateless function over plain data: the front end keeps
//! the choices, calls the engine and renders what it returns. No Rust object
//! crosses the boundary, so the JavaScript side never frees memory by hand.
//! The types crossing it are serialized with `serde`, described to
//! TypeScript by `tsify` and passed as [`Ts`], which turns a value the front
//! end got wrong into a thrown error instead of a panic.

use co_kernel::{Characteristic, ViolationReport};
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

/// The engine a character is built with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    /// COC2, `co-contemporain`.
    Contemporain,
    /// COF2, `co-fantasy`.
    Fantasy,
}

/// The violations of a character, in the engine's stable order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct Report {
    /// Every violation, waived or not.
    pub violations: Vec<ReportedViolation>,
}

/// One violation of a [`Report`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
pub struct ReportedViolation {
    /// The stable rule identifier, such as `COF.CRE.PROFIL`.
    pub rule: String,
    /// The mechanic that emitted the violation.
    pub mechanic: String,
    /// What the violation is about.
    pub subject: Subject,
    /// Its kind.
    pub category: Category,
    /// Whether a waiver accepts it.
    pub waived: bool,
}

/// What a violation is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Tsify)]
#[serde(tag = "kind", content = "id", rename_all = "lowercase")]
pub enum Subject {
    /// The character as a whole.
    Character,
    /// One characteristic, by its abbreviation (`AGI`, ...).
    Characteristic(String),
    /// One voie.
    Voie(String),
    /// One ability.
    Ability(String),
    /// A derived value or a budget of the engine.
    Value(String),
}

/// The kind of a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Category {
    /// A required choice is missing.
    Incomplete,
    /// A rule is broken.
    Deviation {
        /// How strongly the book states the rule.
        severity: Severity,
    },
    /// A value cannot be computed from the current inputs.
    Uncomputable,
    /// Information only.
    Notice,
}

/// How strongly the book states a broken rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The book calls the rule indicative.
    Recommended,
    /// A book rule the Game Master may waive.
    Standard,
    /// The book says never.
    Absolute,
}

/// The report of a character of `engine` with no choice made yet.
///
/// Characters do not store their choices yet, so this is the report of
/// every character the backend holds.
///
/// # Errors
///
/// Throws when `engine` is not an engine name.
#[wasm_bindgen(js_name = newCharacterReport)]
pub fn new_character_report_js(engine: &Ts<Engine>) -> Result<Ts<Report>, JsError> {
    Ok(new_character_report(engine.to_rust()?).into_ts()?)
}

/// The report of a character of `engine` with no choice made yet.
#[must_use]
pub fn new_character_report(engine: Engine) -> Report {
    let report = match engine {
        Engine::Contemporain => co_contemporain::Character::new().report(),
        Engine::Fantasy => co_fantasy::Character::new().report(),
    };
    Report::from(&report)
}

impl From<&ViolationReport> for Report {
    fn from(report: &ViolationReport) -> Self {
        let violations = report
            .entries()
            .iter()
            .map(|entry| {
                let violation = entry.violation();
                ReportedViolation {
                    rule: violation.rule().as_str().to_owned(),
                    mechanic: violation.mechanic().as_str().to_owned(),
                    subject: violation.subject().into(),
                    category: violation.category().into(),
                    waived: entry.is_waived(),
                }
            })
            .collect();
        Self { violations }
    }
}

impl From<&co_kernel::Subject> for Subject {
    fn from(subject: &co_kernel::Subject) -> Self {
        match subject {
            co_kernel::Subject::Character => Self::Character,
            co_kernel::Subject::Characteristic(characteristic) => {
                Self::Characteristic(abbreviation(*characteristic).to_owned())
            }
            co_kernel::Subject::Voie(voie) => Self::Voie(voie.as_str().to_owned()),
            co_kernel::Subject::Ability(ability) => Self::Ability(ability.as_str().to_owned()),
            co_kernel::Subject::Value(value) => Self::Value(value.as_str().to_owned()),
        }
    }
}

impl From<co_kernel::Category> for Category {
    fn from(category: co_kernel::Category) -> Self {
        match category {
            co_kernel::Category::Incomplete => Self::Incomplete,
            co_kernel::Category::Deviation(severity) => Self::Deviation {
                severity: severity.into(),
            },
            co_kernel::Category::Uncomputable => Self::Uncomputable,
            co_kernel::Category::Notice => Self::Notice,
        }
    }
}

impl From<co_kernel::Severity> for Severity {
    fn from(severity: co_kernel::Severity) -> Self {
        match severity {
            co_kernel::Severity::Recommended => Self::Recommended,
            co_kernel::Severity::Standard => Self::Standard,
            co_kernel::Severity::Absolute => Self::Absolute,
        }
    }
}

/// The abbreviation the books use for `characteristic`.
const fn abbreviation(characteristic: Characteristic) -> &'static str {
    match characteristic {
        Characteristic::Agi => "AGI",
        Characteristic::Con => "CON",
        Characteristic::For => "FOR",
        Characteristic::Per => "PER",
        Characteristic::Cha => "CHA",
        Characteristic::Int => "INT",
        Characteristic::Vol => "VOL",
    }
}

#[cfg(test)]
mod tests {
    use co_kernel::{
        Author, Mechanic, RuleId, Subject as KernelSubject, Violation, ViolationReport, Waiver,
    };

    use super::*;

    const RULE: RuleId = RuleId::from_static("TEST.CRE.LIMIT");
    const MECHANIC: Mechanic = Mechanic::new("test-mechanic");

    #[test]
    fn new_characters_of_each_engine_are_incomplete() {
        for engine in [Engine::Contemporain, Engine::Fantasy] {
            let report = new_character_report(engine);
            assert!(!report.violations.is_empty(), "{engine:?}");
            assert!(
                report
                    .violations
                    .iter()
                    .any(|violation| violation.category == Category::Incomplete),
                "{engine:?}"
            );
        }
    }

    #[test]
    fn reports_violations_in_the_engine_order() {
        let report = co_fantasy::Character::new().report();
        let rules: Vec<&str> = report
            .entries()
            .iter()
            .map(|entry| entry.violation().rule().as_str())
            .collect();
        let mapped = Report::from(&report);
        let mapped_rules: Vec<&str> = mapped.violations.iter().map(|v| v.rule.as_str()).collect();
        assert_eq!(mapped_rules, rules);
    }

    #[test]
    fn maps_a_waived_deviation() {
        let subject = KernelSubject::Characteristic(Characteristic::For);
        let violation = Violation::deviation(
            RULE,
            MECHANIC,
            subject.clone(),
            co_kernel::Severity::Absolute,
        );
        let waiver = Waiver::new(RULE, subject, Author::new("gm"), "test reason");
        let report = Report::from(&ViolationReport::new(vec![violation], &[waiver]));
        assert_eq!(
            report.violations,
            vec![ReportedViolation {
                rule: "TEST.CRE.LIMIT".into(),
                mechanic: "test-mechanic".into(),
                subject: Subject::Characteristic("FOR".into()),
                category: Category::Deviation {
                    severity: Severity::Absolute,
                },
                waived: true,
            }]
        );
    }

    #[test]
    fn maps_every_subject() {
        let subjects = [
            (KernelSubject::Character, Subject::Character),
            (
                KernelSubject::Voie(co_kernel::VoieId::new("test-voie")),
                Subject::Voie("test-voie".into()),
            ),
            (
                KernelSubject::Ability(co_kernel::AbilityId::new("test-ability")),
                Subject::Ability("test-ability".into()),
            ),
            (
                KernelSubject::Value(co_kernel::ValueId::from_static("test-value")),
                Subject::Value("test-value".into()),
            ),
        ];
        for (kernel, expected) in subjects {
            assert_eq!(Subject::from(&kernel), expected);
        }
    }

    #[test]
    fn maps_every_category() {
        assert_eq!(
            Category::from(co_kernel::Category::Incomplete),
            Category::Incomplete
        );
        assert_eq!(
            Category::from(co_kernel::Category::Uncomputable),
            Category::Uncomputable
        );
        assert_eq!(
            Category::from(co_kernel::Category::Notice),
            Category::Notice
        );
        assert_eq!(
            Category::from(co_kernel::Category::Deviation(
                co_kernel::Severity::Recommended
            )),
            Category::Deviation {
                severity: Severity::Recommended,
            }
        );
    }
}
