//! Chroniques Oubliées Contemporain, second edition (COC2): character
//! creation and the experience spent after it, core rules only.
//!
//! A [`Character`] holds the creation choices (age, domaine, voie ranks,
//! traits, characteristic points, items carried, protections worn), the XP
//! earned and each [`XpSpending`], the characteristic each rank 5 raises,
//! the sequels acquired in play, and the Game Master's adjustments and
//! waivers. Every other value is computed with
//! its breakdown, and every broken rule is reported as a violation. Rule
//! identifiers trace to the co-rules skill.
//!
//! ```
//! use co_contemporain::{Character, Domaine, Trait, Value, Voie, Weapon};
//! use co_kernel::{Characteristic, Dice};
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
//! // Bare hands: 1d4 + FOR 2.
//! assert_eq!(character.damage_dice(Weapon::MainsNues), Dice::new([4]));
//! let damage = character.value(Value::Damage(Weapon::MainsNues));
//! assert_eq!(damage.map(|v| v.total()), Some(2));
//! assert!(character.report().is_clean());
//! ```

mod age;
mod character;
mod characteristics;
mod domaine;
mod equipment;
mod experience;
mod health;
mod label;
mod play;
mod protection;
mod rules;
mod secondary;
mod step;
mod traits;
mod voie;
mod weapon;

pub use age::{Age, AgeBracket};
pub use character::{AttackMode, Character, Target, Value};
pub use characteristics::ALLOCATED_POINTS;
pub use domaine::Domaine;
pub use equipment::Item;
pub use experience::{Experience, SpendingOffer, XpSpending, xp_cost};
pub use health::{HEALTH_SCALE_STEPS, HealthState};
pub use label::{label, term_label};
pub use play::{Equipment, InPlay};
pub use protection::Protection;
pub use step::{Placement, SpendingStep, Step};
pub use traits::{Sequel, SocialClass, Trait, TraitPoints};
pub use voie::Voie;
pub use weapon::Weapon;
