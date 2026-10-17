//! Creating an account (ADR BE-FIRST-SIGN-IN): the first sign-in of an
//! identity creates no user. It starts a registration, kept for its browser,
//! that the person completes with their nickname and profile, or cancels,
//! leaving the identity free.

use std::time::Duration;

use crate::{
    Nickname, ProviderSummary, TokenHash,
    session::{hash_token, random_token},
};

/// How long a registration waits for its nickname and profile.
pub const REGISTRATION_TTL: Duration = Duration::from_mins(30);

/// The secret a browser presents to complete or cancel its registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationToken(String);

impl RegistrationToken {
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

    /// What the repository keeps: reading it gives no usable registration.
    pub(crate) fn hash(&self) -> TokenHash {
        hash_token(&self.0)
    }
}

/// A registration in progress: the identity a provider vouched for, which
/// no user has yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registration {
    /// The issuer of the identity.
    pub issuer: String,
    /// The subject of the identity.
    pub subject: String,
    /// The private label the identity will have.
    pub label: String,
    /// The nickname suggested, free when the registration started.
    pub nickname: Nickname,
}

/// A registration in progress, with the provider of its identity if it is
/// still configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRegistration {
    /// The registration.
    pub registration: Registration,
    /// The provider of its issuer, unless no longer configured.
    pub provider: Option<ProviderSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_and_hashed() {
        let token = RegistrationToken::random();
        assert_eq!(token.as_str().len(), 64);
        assert_ne!(token, RegistrationToken::random());
        assert_eq!(
            token.hash(),
            RegistrationToken::from_cookie(token.as_str()).hash()
        );
        assert_ne!(token.hash().0, token.as_str());
    }
}
