//! Concepts shared by every Chroniques Oubliées engine.
//!
//! An engine computes derived values with their breakdown, emits violations,
//! and matches them against the waivers the Game Master has granted. All
//! identifiers below are invented.
//!
//! ```
//! use co_kernel::{
//!     Adjustment, Author, Characteristic, Characteristics, DerivedValue, Mechanic, RuleId,
//!     Source, Subject, ValueId, Violation, ViolationReport, Waiver,
//! };
//!
//! const BASE_RULE: RuleId = RuleId::from_static("TEST.DER.BASE");
//! const LIMIT_RULE: RuleId = RuleId::from_static("TEST.CRE.LIMIT");
//! const MECHANIC: Mechanic = Mechanic::new("test-mechanic");
//! const VALUE: ValueId = ValueId::from_static("test-value");
//!
//! // Choices: characteristic values.
//! let characteristics = Characteristics::from_fn(|c| match c {
//!     Characteristic::Agi => 2.into(),
//!     Characteristic::For => 6.into(),
//!     _ => 0.into(),
//! });
//!
//! // A derived value: a constant, a characteristic, and a GM adjustment.
//! let adjustment = Adjustment::new(1, Author::new("gm"), "test reason");
//! let value = DerivedValue::new()
//!     .with(Source::Rule(BASE_RULE), 10)
//!     .with_characteristic(&characteristics, Characteristic::Agi)
//!     .with_adjustment(&adjustment);
//! assert_eq!(value.total(), 13);
//!
//! // Violations: the adjustment notices, and a deviation found by a mechanic.
//! let subject = Subject::Value(VALUE);
//! let mut violations: Vec<Violation> = value.adjustment_notices(&subject).collect();
//! let strength = characteristics[Characteristic::For].get();
//! violations.push(
//!     Violation::deviation(
//!         LIMIT_RULE,
//!         MECHANIC,
//!         Subject::Characteristic(Characteristic::For),
//!     )
//!     .with_parameter("value", strength)
//!     .with_parameter("max", 5),
//! );
//!
//! // The GM accepted the deviation.
//! let waiver = Waiver::new(
//!     LIMIT_RULE,
//!     Subject::Characteristic(Characteristic::For),
//!     Author::new("gm"),
//!     "test reason",
//! );
//!
//! let report = ViolationReport::new(violations, &[waiver]);
//! assert!(report.is_clean());
//! assert_eq!(report.notices().count(), 1);
//! assert!(report.unused_waivers().is_empty());
//! ```

mod adjustment;
mod characteristic;
mod derived;
mod report;
mod violation;
mod voie;
mod waiver;

pub use adjustment::{Adjustment, Author};
pub use characteristic::{Characteristic, CharacteristicValue, Characteristics};
pub use derived::{Contribution, DerivedValue, Source};
pub use report::{ReportEntry, ViolationReport};
pub use violation::{
    ADJUSTMENT_RULE, Category, Mechanic, Parameter, ParameterValue, RuleId, Subject, ValueId,
    Violation, adjustment_notice,
};
pub use voie::{AbilityId, Rank, Voie, VoieId};
pub use waiver::Waiver;
