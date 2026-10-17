//! Sessions (ADR 0009): a random token the browser keeps, of which the
//! server keeps only the hash.

use std::{fmt::Write as _, time::Duration};

use sha2::{Digest, Sha256};

/// How long a session lasts.
pub const SESSION_TTL: Duration = Duration::from_hours(30 * 24);

/// The secret a browser presents to be signed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    /// A new random token.
    pub(crate) fn random() -> Self {
        Self(random_token())
    }

    /// The token a browser sent.
    #[must_use]
    pub fn from_cookie(value: &str) -> Self {
        Self(value.to_owned())
    }

    /// The token, to send to the browser.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// What the repository keeps: reading it gives no usable session.
    pub(crate) fn hash(&self) -> TokenHash {
        TokenHash(hex(&Sha256::digest(self.0.as_bytes())))
    }
}

/// The SHA-256 of a session token, in hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenHash(pub(crate) String);

/// 32 random bytes, in hex.
pub(crate) fn random_token() -> String {
    hex(&rand::random::<[u8; 32]>())
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_and_hashed() {
        let token = SessionToken::random();
        assert_eq!(token.as_str().len(), 64);
        assert_ne!(token, SessionToken::random());
        assert_eq!(
            token.hash(),
            SessionToken::from_cookie(token.as_str()).hash()
        );
        assert_ne!(token.hash().0, token.as_str());
    }
}
