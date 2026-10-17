//! Signing in with OIDC providers (ADR 0009) and linking identities
//! (ADR 0010): the sign-in in progress between the redirect to a provider
//! and its callback, the identity the provider vouches for, and how a
//! sign-in ends.

use std::{fmt, time::Duration};

use crate::{Identity, SessionToken, UserId, session::random_token};

/// How long a sign-in in progress waits for its callback.
pub const SIGN_IN_TTL: Duration = Duration::from_mins(10);
/// The nickname a new user wants when the provider gives none.
const DEFAULT_NICKNAME: &str = "Joueur";

/// The id of a configured provider, used in URLs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderId(String);

impl ProviderId {
    /// The id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ProviderId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for ProviderId {
    fn from(id: &str) -> Self {
        Self(id.to_owned())
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A configured provider, as users see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSummary {
    /// Its id, used in URLs.
    pub id: ProviderId,
    /// Its display name.
    pub name: String,
}

/// The id of a sign-in in progress, kept by the browser that started it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInId(String);

impl SignInId {
    pub(crate) fn random() -> Self {
        Self(random_token())
    }

    /// The id a browser sent.
    #[must_use]
    pub fn from_cookie(value: &str) -> Self {
        Self(value.to_owned())
    }

    /// The id, to send to the browser.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What redirecting to a provider asks of it: where to send the browser,
/// and the secrets the callback is checked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authorization {
    /// The provider's authorization URL, with the request.
    pub url: String,
    /// The `state` sent, to be given back.
    pub state: String,
    /// The `nonce` the ID token must hold.
    pub nonce: String,
    /// The PKCE verifier of the challenge sent.
    pub pkce_verifier: String,
}

/// A sign-in in progress, between the redirect to a provider and its
/// callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSignIn {
    /// The provider signed in with.
    pub provider: ProviderId,
    /// The `state` sent to the provider, to be given back.
    pub state: String,
    /// The `nonce` the ID token must hold.
    pub nonce: String,
    /// The PKCE verifier of the challenge sent to the provider.
    pub pkce_verifier: String,
    /// The user the identity is linked to, when linking one rather than
    /// signing in.
    pub link: Option<UserId>,
}

/// A sign-in started: the browser keeps `id` and goes to `url`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartedSignIn {
    /// The id of the sign-in in progress.
    pub id: SignInId,
    /// The provider's authorization URL.
    pub url: String,
}

/// What a provider sends back to the callback.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProviderAnswer {
    /// The authorization code, unless it failed.
    pub code: Option<String>,
    /// The `state` it was given.
    pub state: Option<String>,
    /// Why it refused, if it did.
    pub error: Option<String>,
}

/// An identity a provider vouched for, with what its ID token says of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedIdentity {
    /// The issuer of the ID token.
    pub issuer: String,
    /// The subject of the ID token.
    pub subject: String,
    /// The `preferred_username` claim.
    pub preferred_username: Option<String>,
    /// The `name` claim.
    pub name: Option<String>,
    /// The `email` claim.
    pub email: Option<String>,
}

impl VerifiedIdentity {
    /// The nickname a new user wants: the `preferred_username` or `name`
    /// claim, else "Joueur". Never the email: a nickname is public.
    pub(crate) fn wanted_nickname(&self) -> &str {
        first([&self.preferred_username, &self.name]).unwrap_or(DEFAULT_NICKNAME)
    }

    /// The private label of the identity: the `email`, `preferred_username`
    /// or `name` claim, or nothing.
    pub(crate) fn label(&self) -> &str {
        first([&self.email, &self.preferred_username, &self.name]).unwrap_or_default()
    }
}

/// The first of `claims` that is not blank, trimmed.
fn first<'a>(claims: impl IntoIterator<Item = &'a Option<String>>) -> Option<&'a str> {
    claims
        .into_iter()
        .flatten()
        .map(|claim| claim.trim())
        .find(|claim| !claim.is_empty())
}

/// What linking an identity to its user did, when it did not fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linking {
    /// Linked: the user can now sign in with it.
    Linked,
    /// Already one of the user's identities.
    AlreadyYours,
}

/// How a sign-in ended, when it did not fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignInEnd {
    /// Signed in: the session opened.
    SignedIn(SessionToken),
    /// An identity linked to the user who started the sign-in.
    Linked(Linking),
}

/// An identity of a user, with its provider if it is still configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedIdentity {
    /// The identity.
    pub identity: Identity,
    /// The provider of its issuer, unless no longer configured.
    pub provider: Option<ProviderSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(
        username: Option<&str>,
        name: Option<&str>,
        email: Option<&str>,
    ) -> VerifiedIdentity {
        VerifiedIdentity {
            issuer: "https://issuer.test".into(),
            subject: "a".into(),
            preferred_username: username.map(Into::into),
            name: name.map(Into::into),
            email: email.map(Into::into),
        }
    }

    #[test]
    fn the_nickname_is_never_the_email() {
        let full = identity(Some(" ana "), Some("Ana A."), Some("ana@mail.test"));
        assert_eq!(full.wanted_nickname(), "ana");
        assert_eq!(full.label(), "ana@mail.test");
        let named = identity(Some(" "), Some("Ana A."), None);
        assert_eq!(named.wanted_nickname(), "Ana A.");
        assert_eq!(named.label(), "Ana A.");
        let email_only = identity(None, None, Some("ana@mail.test"));
        assert_eq!(email_only.wanted_nickname(), "Joueur");
        assert_eq!(email_only.label(), "ana@mail.test");
        assert_eq!(identity(None, None, None).label(), "");
    }
}
