//! Characters (ADR APP-DOCUMENT): a name and the choices document of an engine,
//! versioned so that a stale replacement is refused.

use std::fmt;

use palimpseste_document::{Document, Engine};

use crate::Error;

/// Identifier of a character, unique across all users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharacterId(i64);

impl CharacterId {
    /// The id as a number, as URLs carry it.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<i64> for CharacterId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl fmt::Display for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The version of a character's document, incremented by each
/// replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(i64);

impl Version {
    /// The version of a new character.
    pub const FIRST: Self = Self(1);

    /// The version as a number.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    /// The version after this one.
    pub(crate) const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<i64> for Version {
    fn from(version: i64) -> Self {
        Self(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The name of a character: trimmed, not blank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterName(pub(crate) String);

impl CharacterName {
    /// `value` trimmed.
    ///
    /// # Errors
    ///
    /// [`Error::BlankName`] when it is blank.
    pub fn parse(value: &str) -> Result<Self, Error> {
        let value = value.trim();
        if value.is_empty() {
            Err(Error::BlankName)
        } else {
            Ok(Self(value.to_owned()))
        }
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A character of a user, tied to its engine, with its choices document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    /// Its id.
    pub id: CharacterId,
    /// Its name.
    pub name: CharacterName,
    /// Its engine, the one of its document.
    pub engine: Engine,
    /// Its choices document, stored as sent (ADR BE-CHECKS).
    pub document: Document,
    /// The version of its document: the `ETag`.
    pub version: Version,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_and_not_blank() {
        assert_eq!(CharacterName::parse(" Brune ").unwrap().as_str(), "Brune");
        assert!(matches!(CharacterName::parse(" "), Err(Error::BlankName)));
    }
}
