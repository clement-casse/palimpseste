//! Chroniques Oubliées Fantasy, second edition (COF2): character creation at
//! level 1, core rules only.
//!
//! A [`Character`] holds the creation choices (profil, peuple, value series
//! and its assignment, peuple modifiers, voie ranks, armour and shield) with
//! the Game Master's adjustments and waivers. Every other value is computed
//! with its breakdown, and every broken rule is reported as a violation.
//! Rule identifiers trace to the co-rules skill.
//!
//! ```
//! use co_fantasy::{Armour, Character, Peuple, Profil, Series, Shield, Value, Voie};
//! use co_kernel::Characteristic;
//!
//! let character = Character::new()
//!     .with_profil(Profil::Guerrier)
//!     .with_peuple(Peuple::Nain)
//!     .with_series(Series::Polyvalent)
//!     .with_series_value(Characteristic::For, 2)
//!     .with_series_value(Characteristic::Con, 2)
//!     .with_series_value(Characteristic::Agi, 2)
//!     .with_series_value(Characteristic::Vol, 1)
//!     .with_series_value(Characteristic::Per, 1)
//!     .with_series_value(Characteristic::Int, 0)
//!     .with_series_value(Characteristic::Cha, -1)
//!     .with_peuple_bonus(Characteristic::Con)
//!     .with_rank(Voie::Bouclier, 1)
//!     .with_rank(Voie::Soldat, 1)
//!     .with_rank(Voie::Nain, 1)
//!     .with_armour(Armour::ChemiseDeMailles)
//!     .with_shield(Shield::GrandBouclier);
//!
//! // The nain raises CON and lowers AGI.
//! assert_eq!(character.characteristics()[Characteristic::Con].get(), 3);
//! // PV = 2 x 5 (combattants) + CON 3.
//! let vigour = character.value(Value::VigourPoints);
//! assert_eq!(vigour.map(|v| v.total()), Some(13));
//! // DEF = 10 + AGI 1 + chemise de mailles 4 + grand bouclier 2.
//! let defence = character.value(Value::Defence);
//! assert_eq!(defence.map(|v| v.total()), Some(17));
//! assert!(character.report().is_clean());
//! ```

mod armour;
mod character;
mod characteristics;
mod derived;
mod peuple;
mod profil;
mod rules;
mod voie;

pub use armour::{Armour, Shield};
pub use character::{AttackMode, Character, Target, Value};
pub use characteristics::{SERIES_VALUE, Series};
pub use peuple::Peuple;
pub use profil::{Family, Profil, RecoveryDie};
pub use voie::Voie;
