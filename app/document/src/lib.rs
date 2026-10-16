//! The choices document of a character (ADR 0007).
//!
//! A [`Document`] holds what the player chose, with the adjustments and
//! waivers granted on it, for one engine. Content is named by the engines'
//! string identifiers (`cof.epee-longue`, `coc.crime`), so the engines'
//! content enums are not mirrored here. A document converts one way, to the
//! engine's `Character`: the document is the source of truth.
//!
//! Converting is also how a document is checked: a document converts unless
//! it is unreadable (unsupported version, unknown identifier, missing author
//! or reason). Breaking a rule never makes it unreadable; violations are
//! computed from the converted character.
//!
//! ```
//! use palimpseste_document::{Document, Engine};
//!
//! let document: Document = serde_json::from_str(
//!     r#"{"engine": "fantasy", "version": 1, "profil": "cof.profil.guerrier"}"#,
//! )?;
//! assert_eq!(document.engine(), Engine::Fantasy);
//! assert!(document.check().is_ok());
//! # Ok::<(), serde_json::Error>(())
//! ```

mod contemporain;
mod error;
mod fantasy;

use co_kernel::{AbilityId, Adjustment, Author, RuleId, ValueId, VoieId, Waiver};
pub use contemporain::{ContemporainDocument, TraitEntry};
pub use error::DocumentError;
pub use fantasy::FantasyDocument;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts")]
use tsify::Tsify;

/// The document version this crate reads and writes.
pub const VERSION: u32 = 1;

/// The engine a character is built with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    /// COC2, `co-contemporain`.
    Contemporain,
    /// COF2, `co-fantasy`.
    Fantasy,
}

/// The choices of a character, for its engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(tag = "engine", rename_all = "lowercase")]
pub enum Document {
    /// A COC2 character.
    Contemporain(ContemporainDocument),
    /// A COF2 character.
    Fantasy(FantasyDocument),
}

impl Document {
    /// The document of a character of `engine` with no choice made.
    #[must_use]
    pub fn empty(engine: Engine) -> Self {
        match engine {
            Engine::Contemporain => Self::Contemporain(ContemporainDocument::empty()),
            Engine::Fantasy => Self::Fantasy(FantasyDocument::empty()),
        }
    }

    /// The engine of the document.
    #[must_use]
    pub const fn engine(&self) -> Engine {
        match self {
            Self::Contemporain(_) => Engine::Contemporain,
            Self::Fantasy(_) => Engine::Fantasy,
        }
    }

    /// Checks that the document can be read, by converting it to its
    /// engine's character. Rules are not checked.
    ///
    /// # Errors
    ///
    /// Returns why the document cannot be read.
    pub fn check(&self) -> Result<(), DocumentError> {
        match self {
            Self::Contemporain(document) => document.to_character().map(|_| ()),
            Self::Fantasy(document) => document.to_character().map(|_| ()),
        }
    }
}

/// One of the seven characteristics, by its abbreviation (`AGI`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub enum Characteristic {
    /// Agilité.
    #[serde(rename = "AGI")]
    Agi,
    /// Constitution.
    #[serde(rename = "CON")]
    Con,
    /// Force.
    #[serde(rename = "FOR")]
    For,
    /// Perception.
    #[serde(rename = "PER")]
    Per,
    /// Charisme.
    #[serde(rename = "CHA")]
    Cha,
    /// Intelligence.
    #[serde(rename = "INT")]
    Int,
    /// Volonté.
    #[serde(rename = "VOL")]
    Vol,
}

impl From<Characteristic> for co_kernel::Characteristic {
    fn from(characteristic: Characteristic) -> Self {
        match characteristic {
            Characteristic::Agi => Self::Agi,
            Characteristic::Con => Self::Con,
            Characteristic::For => Self::For,
            Characteristic::Per => Self::Per,
            Characteristic::Cha => Self::Cha,
            Characteristic::Int => Self::Int,
            Characteristic::Vol => Self::Vol,
        }
    }
}

impl From<co_kernel::Characteristic> for Characteristic {
    fn from(characteristic: co_kernel::Characteristic) -> Self {
        match characteristic {
            co_kernel::Characteristic::Agi => Self::Agi,
            co_kernel::Characteristic::Con => Self::Con,
            co_kernel::Characteristic::For => Self::For,
            co_kernel::Characteristic::Per => Self::Per,
            co_kernel::Characteristic::Cha => Self::Cha,
            co_kernel::Characteristic::Int => Self::Int,
            co_kernel::Characteristic::Vol => Self::Vol,
        }
    }
}

/// One value per characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(default)]
pub struct PerCharacteristic<T> {
    /// Agilité.
    #[serde(rename = "AGI")]
    pub agi: T,
    /// Constitution.
    #[serde(rename = "CON")]
    pub con: T,
    /// Force.
    #[serde(rename = "FOR")]
    pub r#for: T,
    /// Perception.
    #[serde(rename = "PER")]
    pub per: T,
    /// Charisme.
    #[serde(rename = "CHA")]
    pub cha: T,
    /// Intelligence.
    #[serde(rename = "INT")]
    pub int: T,
    /// Volonté.
    #[serde(rename = "VOL")]
    pub vol: T,
}

impl<T: Copy> PerCharacteristic<T> {
    /// The value of `characteristic`.
    pub const fn get(&self, characteristic: co_kernel::Characteristic) -> T {
        match characteristic {
            co_kernel::Characteristic::Agi => self.agi,
            co_kernel::Characteristic::Con => self.con,
            co_kernel::Characteristic::For => self.r#for,
            co_kernel::Characteristic::Per => self.per,
            co_kernel::Characteristic::Cha => self.cha,
            co_kernel::Characteristic::Int => self.int,
            co_kernel::Characteristic::Vol => self.vol,
        }
    }
}

/// The ranks acquired in a voie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct VoieEntry {
    /// The voie's identifier.
    pub id: String,
    /// The ranks acquired, gaps and out-of-bound ranks included.
    pub ranks: Vec<u8>,
}

/// An item carried.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ItemEntry {
    /// A weapon of the engine's weapon tables.
    Weapon {
        /// The weapon's identifier.
        id: String,
        /// How many are carried.
        quantity: u32,
    },
    /// Any other item, named freely.
    Other {
        /// The item's name.
        name: String,
        /// How many are carried.
        quantity: u32,
    },
}

/// What a manual adjustment applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(tag = "kind", content = "id", rename_all = "lowercase")]
pub enum Target {
    /// A starting characteristic.
    Characteristic(Characteristic),
    /// A value of the engine, by its identifier.
    Value(String),
}

/// A manual adjustment, attributed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct AdjustmentEntry {
    /// What the adjustment applies to.
    pub target: Target,
    /// The signed amount added.
    pub amount: i32,
    /// Who made the adjustment.
    pub author: String,
    /// Why.
    pub reason: String,
}

impl AdjustmentEntry {
    fn adjustment(&self) -> Result<Adjustment, DocumentError> {
        Ok(Adjustment::new(
            self.amount,
            author(&self.author)?,
            reason(&self.reason)?,
        ))
    }
}

/// What a violation or a waiver is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
#[serde(tag = "kind", content = "id", rename_all = "lowercase")]
pub enum Subject {
    /// The character as a whole.
    Character,
    /// One characteristic.
    Characteristic(Characteristic),
    /// One voie.
    Voie(String),
    /// One ability.
    Ability(String),
    /// A value, a budget or a content choice of the engine.
    Value(String),
}

impl From<&co_kernel::Subject> for Subject {
    fn from(subject: &co_kernel::Subject) -> Self {
        match subject {
            co_kernel::Subject::Character => Self::Character,
            co_kernel::Subject::Characteristic(c) => Self::Characteristic((*c).into()),
            co_kernel::Subject::Voie(voie) => Self::Voie(voie.as_str().to_owned()),
            co_kernel::Subject::Ability(ability) => Self::Ability(ability.as_str().to_owned()),
            co_kernel::Subject::Value(value) => Self::Value(value.as_str().to_owned()),
        }
    }
}

impl Subject {
    fn to_kernel(&self) -> Result<co_kernel::Subject, DocumentError> {
        Ok(match self {
            Self::Character => co_kernel::Subject::Character,
            Self::Characteristic(c) => co_kernel::Subject::Characteristic((*c).into()),
            Self::Voie(id) => co_kernel::Subject::Voie(VoieId::new(filled("subject", id)?)),
            Self::Ability(id) => {
                co_kernel::Subject::Ability(AbilityId::new(filled("subject", id)?))
            }
            Self::Value(id) => co_kernel::Subject::Value(ValueId::new(filled("subject", id)?)),
        })
    }
}

/// A waiver granted on a deviation, attributed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(Tsify))]
pub struct WaiverEntry {
    /// The rule identifier of the deviation, such as `COF.CRE.EQUIPMENT`.
    pub rule: String,
    /// What the deviation is about.
    pub subject: Subject,
    /// Who granted the waiver.
    pub author: String,
    /// Why.
    pub reason: String,
}

impl WaiverEntry {
    fn waiver(&self) -> Result<Waiver, DocumentError> {
        Ok(Waiver::new(
            RuleId::new(filled("rule", &self.rule)?),
            self.subject.to_kernel()?,
            author(&self.author)?,
            reason(&self.reason)?,
        ))
    }
}

/// Fails unless the document is of [`VERSION`].
const fn check_version(version: u32) -> Result<(), DocumentError> {
    if version == VERSION {
        Ok(())
    } else {
        Err(DocumentError::UnsupportedVersion(version))
    }
}

/// Finds the content `id` names with `from_id`.
fn content<T>(
    kind: &'static str,
    id: &str,
    from_id: impl Fn(&str) -> Option<T>,
) -> Result<T, DocumentError> {
    from_id(id).ok_or_else(|| DocumentError::UnknownContent {
        kind,
        id: id.to_owned(),
    })
}

/// Fails when `text` is blank.
fn filled<'a>(field: &'static str, text: &'a str) -> Result<&'a str, DocumentError> {
    if text.trim().is_empty() {
        Err(DocumentError::Empty(field))
    } else {
        Ok(text)
    }
}

fn author(author: &str) -> Result<Author, DocumentError> {
    filled("author", author).map(Author::new)
}

fn reason(reason: &str) -> Result<&str, DocumentError> {
    filled("reason", reason)
}
