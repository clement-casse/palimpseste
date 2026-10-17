//! Users (ADR BE-USERS): a private id, a public nickname and profile, and the
//! identities they sign in with.

use std::fmt;

use crate::Error;

/// The longest nickname, in characters.
pub const NICKNAME_MAX: usize = 32;
/// The longest profile, in characters.
pub const PROFILE_MAX: usize = 2000;
/// How many characters of a wanted nickname a new user keeps, leaving room
/// for a number making it unique.
const NICKNAME_BASE_LENGTH: usize = 28;

/// Identifier of a user. Private: it never leaves the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserId(pub(crate) i64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A user, owner of a list of characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Private: never sent out.
    pub id: UserId,
    /// Public, unique regardless of ASCII case.
    pub nickname: Nickname,
    /// Public free text, empty until written.
    pub profile: Profile,
}

/// A public nickname: trimmed, not blank, at most [`NICKNAME_MAX`]
/// characters. Uniqueness is the repository's to check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nickname(pub(crate) String);

impl Nickname {
    /// `value` trimmed.
    ///
    /// # Errors
    ///
    /// [`Error::BlankNickname`] when it is blank, [`Error::NicknameTooLong`]
    /// when it is too long.
    pub fn parse(value: &str) -> Result<Self, Error> {
        let value = value.trim();
        if value.is_empty() {
            Err(Error::BlankNickname)
        } else if value.chars().count() > NICKNAME_MAX {
            Err(Error::NicknameTooLong)
        } else {
            Ok(Self(value.to_owned()))
        }
    }

    /// The nickname as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The nicknames a new user wanting `wanted` may get, in order: `wanted`
/// cut to `NICKNAME_BASE_LENGTH` characters, then `wanted 2`, `wanted 3`,
/// ... The repository gives the first one no user has.
pub(crate) fn nickname_candidates(wanted: &str) -> impl Iterator<Item = Nickname> {
    let base: String = wanted.chars().take(NICKNAME_BASE_LENGTH).collect();
    std::iter::once(Nickname(base.clone()))
        .chain((2_u64..).map(move |number| Nickname(format!("{base} {number}"))))
}

/// A public profile: trimmed free text, at most [`PROFILE_MAX`] characters.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Profile(pub(crate) String);

impl Profile {
    /// `value` trimmed.
    ///
    /// # Errors
    ///
    /// [`Error::ProfileTooLong`] when it is too long.
    pub fn parse(value: &str) -> Result<Self, Error> {
        let value = value.trim();
        if value.chars().count() > PROFILE_MAX {
            Err(Error::ProfileTooLong)
        } else {
            Ok(Self(value.to_owned()))
        }
    }

    /// The profile as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An identity a user signs in with, as its owner sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The issuer of the identity, naming its provider.
    pub issuer: String,
    /// Private: what the provider last said the identity is (an email, a
    /// username), so its owner tells their identities apart; may be empty.
    pub label: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nicknames_are_trimmed_not_blank_and_short() {
        assert_eq!(Nickname::parse(" Anaïs ").unwrap().as_str(), "Anaïs");
        assert!(matches!(Nickname::parse("  "), Err(Error::BlankNickname)));
        assert!(Nickname::parse(&"é".repeat(NICKNAME_MAX)).is_ok());
        assert!(matches!(
            Nickname::parse(&"x".repeat(NICKNAME_MAX + 1)),
            Err(Error::NicknameTooLong)
        ));
    }

    #[test]
    fn profiles_are_trimmed_and_short() {
        assert_eq!(Profile::parse(" Joue. ").unwrap().as_str(), "Joue.");
        assert_eq!(Profile::parse("").unwrap(), Profile::default());
        assert!(matches!(
            Profile::parse(&"x".repeat(PROFILE_MAX + 1)),
            Err(Error::ProfileTooLong)
        ));
    }

    #[test]
    fn candidates_cut_the_wanted_nickname_then_number_it() {
        let candidates: Vec<String> = nickname_candidates("Ana")
            .take(3)
            .map(|nickname| nickname.0)
            .collect();
        assert_eq!(candidates, ["Ana", "Ana 2", "Ana 3"]);
        let long = nickname_candidates(&"x".repeat(40)).next().unwrap();
        assert_eq!(long.0, "x".repeat(NICKNAME_BASE_LENGTH));
    }
}
