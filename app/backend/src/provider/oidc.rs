//! Identity providers speaking `OpenID Connect`, through `openidconnect`:
//! the Authorization Code flow with PKCE, `state` and `nonce`. Discovery
//! runs at each sign-in, so rotated keys need no cache. Tokens, codes and
//! claims are never logged.

use anyhow::{Context as _, anyhow};
use async_trait::async_trait;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest,
};

use crate::{
    Authorization, Error, PendingSignIn, ProviderId, ProviderSummary, VerifiedIdentity,
    provider::IdentityProviders,
};

/// The scopes asked for besides `openid`, unless a provider sets its own.
const DEFAULT_SCOPES: &str = "profile email";

/// A client discovered from a provider's metadata, with its redirect URI.
type Client = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

/// The configuration of an OIDC provider.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    id: ProviderId,
    name: String,
    issuer: IssuerUrl,
    client_id: ClientId,
    client_secret: ClientSecret,
    /// The scopes asked for besides `openid`, which is always asked for.
    scopes: Vec<Scope>,
}

impl ProviderConfig {
    /// A provider known by `id` in URLs and as `name` to users, asked for
    /// the scopes `openid profile email`.
    ///
    /// # Errors
    ///
    /// [`Error::Underlying`] when `id` is not made of lower-case ASCII
    /// letters, digits and dashes, or `issuer` is not a URL.
    pub fn new(
        id: &str,
        name: &str,
        issuer: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<Self, Error> {
        let slug = id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if id.is_empty() || !slug {
            return Err(
                anyhow!("provider id {id:?} is not lower-case letters, digits and dashes").into(),
            );
        }
        let issuer = IssuerUrl::new(issuer.to_owned())
            .with_context(|| format!("issuer of provider {id} is not a URL"))?;
        Ok(Self {
            id: id.into(),
            name: name.to_owned(),
            issuer,
            client_id: ClientId::new(client_id.to_owned()),
            client_secret: ClientSecret::new(client_secret.to_owned()),
            scopes: Vec::new(),
        }
        .with_scopes(DEFAULT_SCOPES))
    }

    /// The same provider, asked for `openid` and the space-separated
    /// `scopes`, for providers lacking the standard ones (Discord has
    /// `identify`, not `profile`).
    #[must_use]
    pub fn with_scopes(mut self, scopes: &str) -> Self {
        self.scopes = scopes
            .split_whitespace()
            // `openid` is always asked for: once is enough.
            .filter(|scope| *scope != "openid")
            .map(|scope| Scope::new(scope.to_owned()))
            .collect();
        self
    }

    fn summary(&self) -> ProviderSummary {
        ProviderSummary {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }
}

/// The configured OIDC providers, the redirect URI of each, and the HTTP
/// client that reaches them.
#[derive(Debug)]
pub struct OidcProviders {
    providers: Vec<(ProviderConfig, RedirectUrl)>,
    http: reqwest::Client,
}

impl OidcProviders {
    /// The `providers`, redirecting back to `public_url`, at
    /// `{public_url}/api/auth/{id}/callback`.
    ///
    /// # Errors
    ///
    /// [`Error::Underlying`] when a provider is configured twice or
    /// `public_url` is not a URL.
    pub fn new(public_url: &str, providers: Vec<ProviderConfig>) -> Result<Self, Error> {
        let public_url = public_url.trim_end_matches('/');
        let mut configured: Vec<(ProviderConfig, RedirectUrl)> = Vec::new();
        for provider in providers {
            if configured.iter().any(|(known, _)| known.id == provider.id) {
                return Err(anyhow!("provider {} is configured twice", provider.id).into());
            }
            let redirect =
                RedirectUrl::new(format!("{public_url}/api/auth/{}/callback", provider.id))
                    .with_context(|| format!("public URL {public_url:?} is not a URL"))?;
            configured.push((provider, redirect));
        }
        let http = reqwest::Client::builder()
            // Following redirects would let a provider make the server
            // reach other hosts.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("building the HTTP client of the providers")?;
        Ok(Self {
            providers: configured,
            http,
        })
    }

    fn provider(&self, id: &ProviderId) -> Result<&(ProviderConfig, RedirectUrl), Error> {
        self.providers
            .iter()
            .find(|(provider, _)| provider.id == *id)
            .ok_or(Error::UnknownProvider)
    }

    /// Discovers `provider` and builds its client.
    async fn client(
        &self,
        (provider, redirect): &(ProviderConfig, RedirectUrl),
    ) -> Result<Client, Error> {
        let metadata = CoreProviderMetadata::discover_async(provider.issuer.clone(), &self.http)
            .await
            .with_context(|| format!("discovering provider {}", provider.id))?;
        Ok(CoreClient::from_provider_metadata(
            metadata,
            provider.client_id.clone(),
            Some(provider.client_secret.clone()),
        )
        .set_redirect_uri(redirect.clone()))
    }
}

#[async_trait]
impl IdentityProviders for OidcProviders {
    fn list(&self) -> Vec<ProviderSummary> {
        self.providers
            .iter()
            .map(|(provider, _)| provider.summary())
            .collect()
    }

    fn provider_of(&self, issuer: &str) -> Option<ProviderSummary> {
        self.providers
            .iter()
            .find(|(provider, _)| provider.issuer.as_str() == issuer)
            .map(|(provider, _)| provider.summary())
    }

    async fn authorize(&self, id: &ProviderId) -> Result<Authorization, Error> {
        let provider = self.provider(id)?;
        let client = self.client(provider).await?;
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scopes(provider.0.scopes.iter().cloned())
            .set_pkce_challenge(challenge)
            .url();
        Ok(Authorization {
            url: url.to_string(),
            state: state.secret().clone(),
            nonce: nonce.secret().clone(),
            pkce_verifier: verifier.secret().clone(),
        })
    }

    async fn verify(
        &self,
        id: &ProviderId,
        code: &str,
        pending: &PendingSignIn,
    ) -> Result<VerifiedIdentity, Error> {
        let client = self.client(self.provider(id)?).await?;
        let response = client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .context("no token endpoint")?
            .set_pkce_verifier(PkceCodeVerifier::new(pending.pkce_verifier.clone()))
            .request_async(&self.http)
            .await
            .context("exchanging the code")?;
        let id_token = response.id_token().context("no ID token")?;
        let claims = id_token
            .claims(
                &client.id_token_verifier(),
                &Nonce::new(pending.nonce.clone()),
            )
            .context("verifying the ID token")?;
        Ok(VerifiedIdentity {
            issuer: claims.issuer().to_string(),
            subject: claims.subject().to_string(),
            preferred_username: claims
                .preferred_username()
                .map(|username| username.to_string()),
            name: claims
                .name()
                .and_then(|name| name.get(None))
                .map(|name| name.to_string()),
            email: claims.email().map(|email| email.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_configuration() {
        assert!(
            ProviderConfig::new("Google", "G", "https://accounts.google.com", "c", "s").is_err()
        );
        assert!(ProviderConfig::new("google", "G", "not a url", "c", "s").is_err());
        let provider = || ProviderConfig::new("dex", "Dex", "https://dex.test", "c", "s").unwrap();
        assert!(OidcProviders::new("http://localhost", vec![provider(), provider()]).is_err());
        assert!(OidcProviders::new("not a url", vec![provider()]).is_err());
        let providers = OidcProviders::new("https://palimpseste.test", vec![provider()]).unwrap();
        assert_eq!(
            providers.list(),
            [ProviderSummary {
                id: "dex".into(),
                name: "Dex".into()
            }]
        );
        assert_eq!(
            providers.provider_of("https://dex.test"),
            Some(provider().summary())
        );
        assert_eq!(providers.provider_of("https://other.test"), None);
    }
}
