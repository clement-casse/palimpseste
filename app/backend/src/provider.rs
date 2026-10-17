//! The identity providers users sign in with (ADR BE-OIDC): what the service
//! asks of them, whatever the protocol.

pub mod oidc;

use async_trait::async_trait;

pub use self::oidc::{OidcProviders, ProviderConfig};
use crate::{Authorization, Error, PendingSignIn, ProviderId, ProviderSummary, VerifiedIdentity};

/// The configured identity providers.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IdentityProviders: Send + Sync + 'static {
    /// The providers, in their configured order.
    fn list(&self) -> Vec<ProviderSummary>;

    /// The provider of identities from `issuer`, if it is configured.
    fn provider_of(&self, issuer: &str) -> Option<ProviderSummary>;

    /// The request that sends a browser to sign in at `provider`;
    /// [`Error::UnknownProvider`] when no provider has this id.
    async fn authorize(&self, provider: &ProviderId) -> Result<Authorization, Error>;

    /// The identity `provider` vouches for in exchange for `code`, checked
    /// against the sign-in in progress `pending` (PKCE, nonce).
    async fn verify(
        &self,
        provider: &ProviderId,
        code: &str,
        pending: &PendingSignIn,
    ) -> Result<VerifiedIdentity, Error>;
}
