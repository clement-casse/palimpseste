//! Chroniques Oubliées Contemporain, second edition (COC2): character
//! creation, core rules only.
//!
//! A [`Character`] holds the creation choices (age, domaine, voie ranks,
//! traits, characteristic points) with the Game Master's adjustments and
//! waivers. Every other value is computed with its breakdown, and every
//! broken rule is reported as a violation. Rule identifiers trace to the
//! co-rules skill.
//!
//! ```
//! use co_contemporain::{Character, Domaine, Trait, Value, Voie};
//! use co_kernel::Characteristic;
//!
//! let character = Character::new()
//!     .with_age(24)
//!     .with_domaine(Domaine::Sport)
//!     .with_rank(Voie::Athletisme, 1)
//!     .with_rank(Voie::Athletisme, 2)
//!     .with_rank(Voie::CombatAMainsNues, 1)
//!     .with_rank(Voie::Vehicules, 1)
//!     .with_trait(Trait::Rapide, 2)
//!     .with_trait(Trait::Superstitieux, 1)
//!     .with_trait(Trait::Ignorant, 1)
//!     .with_carac_points(Characteristic::Agi, 3)
//!     .with_carac_points(Characteristic::For, 2)
//!     .with_carac_points(Characteristic::Con, 1)
//!     .with_carac_points(Characteristic::Per, 2)
//!     .with_carac_points(Characteristic::Vol, 2);
//!
//! // Athlétisme rank 1 adds CON +1 on top of the allocated point.
//! assert_eq!(character.characteristics()[Characteristic::Con].get(), 2);
//! // Init = 10 + INT 0 + PER 2, and Rapide +3.
//! let initiative = character.value(Value::Initiative);
//! assert_eq!(initiative.map(|v| v.total()), Some(15));
//! assert!(character.report().is_clean());
//! ```

mod age;
mod character;
mod characteristics;
mod domaine;
mod rules;
mod secondary;
mod traits;
mod voie;

pub use age::{Age, AgeBracket};
pub use character::{AttackMode, Character, Target, Value};
pub use characteristics::ALLOCATED_POINTS;
pub use domaine::Domaine;
pub use traits::{Sequel, SocialClass, Trait, TraitPoints};
pub use voie::Voie;

/// Converts a count to `i32`, saturating.
fn saturate(count: usize) -> i32 {
    i32::try_from(count).unwrap_or(i32::MAX)
}

/// Clamps an exact sum to the `i32` range.
fn clamp(sum: i64) -> i32 {
    i32::try_from(sum).unwrap_or(if sum < 0 { i32::MIN } else { i32::MAX })
}
