//! Signing in through OIDC providers (ADR 0009).
//!
//! Any number of providers are configured, each by its issuer. Signing in is
//! the Authorization Code flow with PKCE, `state` and `nonce`: `login` keeps
//! the sign-in in the store under a short-lived cookie and redirects to the
//! provider; `callback` takes it back once, checks `state`, exchanges the
//! code, verifies the ID token, finds or creates the user of the identity
//! and opens a session. The session token travels in an `HttpOnly`,
//! `SameSite=Lax` cookie; the store keeps only its hash. Discovery runs at
//! each sign-in, so rotated keys need no cache. Tokens, codes and claims are
//! never logged.

use std::{env, fmt::Write as _, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{AppendHeaders, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreIdTokenClaims, CoreProviderMetadata},
    reqwest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    api::{ApiError, AppState, blocking},
    store::{PendingLogin, User},
};

const SESSION_COOKIE: &str = "palimpseste_session";
const LOGIN_COOKIE: &str = "palimpseste_login";
const LOGIN_COOKIE_PATH: &str = "/api/auth";
const SESSION_TTL: Duration = Duration::from_hours(30 * 24);
const LOGIN_TTL: Duration = Duration::from_mins(10);
/// Where a sign-in ends: the app, told when it failed.
const SIGNED_IN: &str = "/";
const SIGN_IN_FAILED: &str = "/login?signin=failed";
/// The scopes asked for besides `openid`, unless a provider sets its own.
const DEFAULT_SCOPES: &str = "profile email";
/// The name of a user whose ID token gives none.
const DEFAULT_NAME: &str = "Joueur";

/// A client discovered from a provider's metadata, with its redirect URI.
type Client = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

/// An OIDC provider users sign in with.
#[derive(Debug, Clone)]
pub struct Provider {
    id: String,
    name: String,
    issuer: IssuerUrl,
    client_id: ClientId,
    client_secret: ClientSecret,
    /// The scopes asked for besides `openid`, which is always asked for.
    scopes: Vec<Scope>,
}

impl Provider {
    /// A provider known by `id` in URLs and as `name` to users, asked for
    /// the scopes `openid profile email`. Fails when `id` is not made of
    /// lower-case ASCII letters, digits and dashes, or `issuer` is not a
    /// URL.
    pub fn new(
        id: &str,
        name: &str,
        issuer: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<Self, String> {
        let slug = id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if id.is_empty() || !slug {
            return Err(format!(
                "provider id {id:?} is not lower-case letters, digits and dashes"
            ));
        }
        let issuer = IssuerUrl::new(issuer.to_owned())
            .map_err(|error| format!("issuer of provider {id} is not a URL: {error}"))?;
        Ok(Self {
            id: id.to_owned(),
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

    /// Provider `id`, from `PALIMPSESTE_OIDC_<ID>_NAME`, `_ISSUER`,
    /// `_CLIENT_ID`, `_CLIENT_SECRET`, and `_SCOPES` if set.
    fn from_env(id: &str) -> Result<Self, String> {
        let prefix = format!(
            "PALIMPSESTE_OIDC_{}_",
            id.to_ascii_uppercase().replace('-', "_")
        );
        let var = |name: &str| {
            env::var(format!("{prefix}{name}")).map_err(|_| format!("{prefix}{name} is not set"))
        };
        let provider = Self::new(
            id,
            &var("NAME")?,
            &var("ISSUER")?,
            &var("CLIENT_ID")?,
            &var("CLIENT_SECRET")?,
        )?;
        Ok(match var("SCOPES") {
            Ok(scopes) => provider.with_scopes(&scopes),
            Err(_) => provider,
        })
    }
}

/// The providers, the public URL they redirect back to, and the HTTP client
/// that reaches them.
#[derive(Debug)]
pub struct Auth {
    providers: Vec<(Provider, RedirectUrl)>,
    secure: bool,
    http: reqwest::Client,
}

impl Auth {
    /// Signing in with `providers`, served at `public_url`. Cookies are
    /// `Secure` when it is `https`.
    pub fn new(public_url: &str, providers: Vec<Provider>) -> Result<Self, String> {
        let public_url = public_url.trim_end_matches('/');
        let mut configured: Vec<(Provider, RedirectUrl)> = Vec::new();
        for provider in providers {
            if configured.iter().any(|(known, _)| known.id == provider.id) {
                return Err(format!("provider {} is configured twice", provider.id));
            }
            let redirect =
                RedirectUrl::new(format!("{public_url}/api/auth/{}/callback", provider.id))
                    .map_err(|error| format!("public URL {public_url:?} is not a URL: {error}"))?;
            configured.push((provider, redirect));
        }
        let http = reqwest::Client::builder()
            // Following redirects would let a provider make the server
            // reach other hosts.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| format!("cannot build the HTTP client: {error}"))?;
        Ok(Self {
            providers: configured,
            secure: public_url.starts_with("https://"),
            http,
        })
    }

    /// Signing in as configured by the environment:
    /// `PALIMPSESTE_PUBLIC_URL` (default `http://127.0.0.1:3000`) and
    /// `PALIMPSESTE_OIDC_PROVIDERS`, a comma-separated list of provider ids,
    /// each described by its own variables (see [`Provider::from_env`]).
    pub fn from_env() -> Result<Self, String> {
        let public_url =
            env::var("PALIMPSESTE_PUBLIC_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
        let providers = env::var("PALIMPSESTE_OIDC_PROVIDERS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(Provider::from_env)
            .collect::<Result<_, _>>()?;
        Self::new(&public_url, providers)
    }

    /// The ids of the providers, in their configured order.
    pub fn provider_ids(&self) -> Vec<&str> {
        self.providers
            .iter()
            .map(|(provider, _)| provider.id.as_str())
            .collect()
    }

    fn provider(&self, id: &str) -> Option<&(Provider, RedirectUrl)> {
        self.providers
            .iter()
            .find(|(provider, _)| provider.id == id)
    }

    /// Discovers `provider` and builds its client.
    async fn client(
        &self,
        (provider, redirect): &(Provider, RedirectUrl),
    ) -> Result<Client, String> {
        let metadata = CoreProviderMetadata::discover_async(provider.issuer.clone(), &self.http)
            .await
            .map_err(|error| format!("discovery failed: {error}"))?;
        Ok(CoreClient::from_provider_metadata(
            metadata,
            provider.client_id.clone(),
            Some(provider.client_secret.clone()),
        )
        .set_redirect_uri(redirect.clone()))
    }

    /// A `Set-Cookie` value; a zero `max_age` removes the cookie.
    fn set_cookie(&self, name: &str, value: &str, path: &str, max_age: Duration) -> String {
        let secure = if self.secure { "; Secure" } else { "" };
        format!(
            "{name}={value}; Path={path}; Max-Age={}; HttpOnly; SameSite=Lax{secure}",
            max_age.as_secs()
        )
    }
}

/// The routes under `/api/auth`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/providers", get(providers))
        .route("/{provider}/login", get(login))
        .route("/{provider}/callback", get(callback))
        .route("/logout", post(logout))
}

/// The signed-in user of a request: `401` without a live session.
#[derive(Debug)]
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token_hash = cookie(&parts.headers, SESSION_COOKIE)
            .map(hash)
            .ok_or(ApiError::Unauthorized)?;
        blocking(state.store.clone(), move |store| {
            store.session_user(&token_hash)
        })
        .await?
        .map(Self)
        .ok_or(ApiError::Unauthorized)
    }
}

#[derive(Debug, Serialize)]
struct Listed {
    id: String,
    name: String,
}

async fn providers(State(auth): State<Arc<Auth>>) -> Json<Vec<Listed>> {
    Json(
        auth.providers
            .iter()
            .map(|(provider, _)| Listed {
                id: provider.id.clone(),
                name: provider.name.clone(),
            })
            .collect(),
    )
}

/// Starts signing in with a provider: redirects to it, keeping what the
/// callback checks under a cookie of this browser.
async fn login(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let auth = &state.auth;
    let provider = auth.provider(&id).ok_or(ApiError::NotFound)?;
    let scopes = provider.0.scopes.iter().cloned();
    let client = match auth.client(provider).await {
        Ok(client) => client,
        Err(reason) => {
            tracing::warn!(provider = %id, %reason, "sign-in failed");
            return Ok(Redirect::to(SIGN_IN_FAILED).into_response());
        }
    };
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (url, csrf, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scopes(scopes)
        .set_pkce_challenge(challenge)
        .url();
    let pending = PendingLogin {
        provider: id,
        state: csrf.secret().clone(),
        nonce: nonce.secret().clone(),
        pkce_verifier: verifier.secret().clone(),
    };
    let login_id = random_token();
    let cookie = auth.set_cookie(LOGIN_COOKIE, &login_id, LOGIN_COOKIE_PATH, LOGIN_TTL);
    blocking(state.store.clone(), move |store| {
        store.purge_expired()?;
        store.create_login(&login_id, &pending, LOGIN_TTL)
    })
    .await?;
    Ok(([(header::SET_COOKIE, cookie)], Redirect::to(url.as_str())).into_response())
}

/// What a provider sends back to the callback.
#[derive(Debug, Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// Ends signing in with a provider: opens a session, or tells the app it
/// failed. The sign-in in progress is used up either way.
async fn callback(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(callback): Query<Callback>,
    headers: HeaderMap,
) -> Response {
    let clear_login = state
        .auth
        .set_cookie(LOGIN_COOKIE, "", LOGIN_COOKIE_PATH, Duration::ZERO);
    match sign_in(&state, &id, callback, &headers).await {
        Ok(token) => {
            let session = state
                .auth
                .set_cookie(SESSION_COOKIE, &token, "/", SESSION_TTL);
            (
                AppendHeaders([
                    (header::SET_COOKIE, clear_login),
                    (header::SET_COOKIE, session),
                ]),
                Redirect::to(SIGNED_IN),
            )
                .into_response()
        }
        Err(reason) => {
            tracing::warn!(provider = %id, %reason, "sign-in failed");
            (
                [(header::SET_COOKIE, clear_login)],
                Redirect::to(SIGN_IN_FAILED),
            )
                .into_response()
        }
    }
}

/// Checks a callback against the sign-in in progress, and opens a session
/// for the identity of its ID token. Gives the session token, or why it
/// failed.
async fn sign_in(
    state: &AppState,
    id: &str,
    callback: Callback,
    headers: &HeaderMap,
) -> Result<String, String> {
    let auth = &state.auth;
    let provider = auth.provider(id).ok_or("unknown provider")?;
    let login_id = cookie(headers, LOGIN_COOKIE)
        .ok_or("no sign-in in progress in this browser")?
        .to_owned();
    let login = blocking(state.store.clone(), move |store| {
        store.take_login(&login_id)
    })
    .await
    .map_err(|_| "store error")?
    .ok_or("unknown or expired sign-in")?;
    if login.provider != id {
        return Err("sign-in started with another provider".into());
    }
    if callback.state.as_deref() != Some(login.state.as_str()) {
        return Err("state does not match".into());
    }
    let code = match (callback.code, callback.error) {
        (_, Some(error)) => return Err(format!("the provider answered {error}")),
        (None, None) => return Err("no code".into()),
        (Some(code), None) => code,
    };
    let client = auth.client(provider).await?;
    let response = client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|error| format!("no token endpoint: {error}"))?
        .set_pkce_verifier(PkceCodeVerifier::new(login.pkce_verifier))
        .request_async(&auth.http)
        .await
        .map_err(|error| format!("code exchange failed: {error}"))?;
    let id_token = response.id_token().ok_or("no ID token")?;
    let claims = id_token
        .claims(&client.id_token_verifier(), &Nonce::new(login.nonce))
        .map_err(|error| format!("invalid ID token: {error}"))?;
    let issuer = claims.issuer().to_string();
    let subject = claims.subject().to_string();
    let name = display_name(claims);
    let token = random_token();
    let token_hash = hash(&token);
    let (user, created) = blocking(state.store.clone(), move |store| {
        let (user, created) = store.user_for_identity(&issuer, &subject, &name)?;
        store.create_session(user.id, &token_hash, SESSION_TTL)?;
        Ok((user, created))
    })
    .await
    .map_err(|_| "store error")?;
    if created {
        tracing::info!(user_id = %user.id, provider = id, "user created");
    }
    tracing::info!(user_id = %user.id, provider = id, "signed in");
    Ok(token)
}

/// Ends the session of the request, if any, and removes its cookie.
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(token_hash) = cookie(&headers, SESSION_COOKIE).map(hash) {
        blocking(state.store.clone(), move |store| {
            store.delete_session(&token_hash)
        })
        .await?;
        tracing::info!("signed out");
    }
    let clear = state
        .auth
        .set_cookie(SESSION_COOKIE, "", "/", Duration::ZERO);
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, clear)]))
}

/// The name a new user gets: the first of the `name`, `preferred_username`
/// and `email` claims that is not blank.
fn display_name(claims: &CoreIdTokenClaims) -> String {
    [
        claims
            .name()
            .and_then(|name| name.get(None))
            .map(|name| name.as_str()),
        claims
            .preferred_username()
            .map(|username| username.as_str()),
        claims.email().map(|email| email.as_str()),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .find(|name| !name.is_empty())
    .unwrap_or(DEFAULT_NAME)
    .to_owned()
}

/// The value of cookie `name` in the request, if sent.
fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| {
            let (key, value) = pair.trim().split_once('=')?;
            (key == name).then_some(value)
        })
}

/// 32 random bytes, in hex.
fn random_token() -> String {
    hex(&rand::random::<[u8; 32]>())
}

/// The SHA-256 of a session token, in hex: what the store keeps.
fn hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// Opens a session of `user` and gives the `Cookie` header that sends it.
#[cfg(test)]
pub fn session_cookie(store: &crate::store::Store, user: crate::store::UserId) -> String {
    let token = random_token();
    store
        .create_session(user, &hash(&token), SESSION_TTL)
        .unwrap();
    format!("{SESSION_COOKIE}={token}")
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    use axum::{Form, body::Body, http::Request};
    use openidconnect::{
        JsonWebKeyId, PrivateSigningKey,
        core::{CoreIdToken, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey},
        url::Url,
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::{api, store::Store};

    const CLIENT_ID: &str = "palimpseste";

    /// What the issuer granted for a code: the ID token it gives for it.
    #[derive(Debug, Clone)]
    struct Grant {
        subject: String,
        nonce: String,
        challenge: String,
    }

    /// An OIDC issuer on a local port: discovery, keys, and a
    /// token endpoint answering the codes granted to it, checking PKCE.
    struct Issuer {
        url: String,
        grants: Arc<Mutex<HashMap<String, Grant>>>,
    }

    impl Issuer {
        async fn start() -> Self {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let key = Arc::new(
                CoreRsaPrivateSigningKey::from_pem(
                    include_str!("../tests/fixtures/issuer-key.pem"),
                    Some(JsonWebKeyId::new("test".into())),
                )
                .unwrap(),
            );
            let grants = Arc::new(Mutex::new(HashMap::<String, Grant>::new()));
            let discovery = json!({
                "issuer": url,
                "authorization_endpoint": format!("{url}/authorize"),
                "token_endpoint": format!("{url}/token"),
                "jwks_uri": format!("{url}/jwks"),
                "response_types_supported": ["code"],
                "subject_types_supported": ["public"],
                "id_token_signing_alg_values_supported": ["RS256"],
            });
            let jwks = json!({"keys": [key.as_verification_key()]});
            let token = {
                let (url, grants) = (url.clone(), grants.clone());
                move |Form(form): Form<HashMap<String, String>>| async move {
                    let grant = grants.lock().unwrap().remove(&form["code"]);
                    let verifier = PkceCodeVerifier::new(form["code_verifier"].clone());
                    let challenge = PkceCodeChallenge::from_code_verifier_sha256(&verifier);
                    let Some(grant) = grant.filter(|g| g.challenge == challenge.as_str()) else {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": "invalid_grant"})),
                        );
                    };
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let claims: CoreIdTokenClaims = serde_json::from_value(json!({
                        "iss": url,
                        "aud": CLIENT_ID,
                        "exp": now + 300,
                        "iat": now,
                        "sub": grant.subject,
                        "nonce": grant.nonce,
                        "name": format!("{} Mock", grant.subject),
                    }))
                    .unwrap();
                    let id_token = CoreIdToken::new(
                        claims,
                        &*key,
                        CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
                        None,
                        None,
                    )
                    .unwrap();
                    (
                        StatusCode::OK,
                        Json(json!({
                            "access_token": "access",
                            "token_type": "bearer",
                            "id_token": id_token.to_string(),
                        })),
                    )
                }
            };
            let router = Router::new()
                .route(
                    "/.well-known/openid-configuration",
                    get(move || async move { Json(discovery) }),
                )
                .route("/jwks", get(move || async move { Json(jwks) }))
                .route("/token", post(token));
            tokio::spawn(async move { axum::serve(listener, router).await });
            Self { url, grants }
        }

        /// Grants `code` to `subject`, for an ID token holding `nonce`.
        fn grant(&self, code: &str, subject: &str, nonce: &str, challenge: &str) {
            self.grants.lock().unwrap().insert(
                code.into(),
                Grant {
                    subject: subject.into(),
                    nonce: nonce.into(),
                    challenge: challenge.into(),
                },
            );
        }
    }

    fn provider(issuer: &Issuer) -> Provider {
        Provider::new("mock", "Mock", &issuer.url, CLIENT_ID, "secret").unwrap()
    }

    fn app(issuer: &Issuer) -> Router {
        app_with(provider(issuer))
    }

    fn app_with(provider: Provider) -> Router {
        let auth = Auth::new("http://localhost:3000/", vec![provider]).unwrap();
        let dir = std::env::temp_dir().join("palimpseste-backend-auth");
        api::router(Arc::new(Store::in_memory().unwrap()), Arc::new(auth), &dir)
    }

    async fn get_with(app: &Router, uri: &str, cookie: Option<&str>) -> Response {
        let request = cookie
            .into_iter()
            .fold(Request::get(uri), |request, cookie| {
                request.header(header::COOKIE, cookie)
            })
            .body(Body::empty())
            .unwrap();
        app.clone().oneshot(request).await.unwrap()
    }

    /// The `name=value` of each cookie a response sets.
    fn set_cookies(response: &Response) -> Vec<String> {
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|value| {
                let value = value.to_str().unwrap();
                value.split(';').next().unwrap().to_owned()
            })
            .collect()
    }

    fn location(response: &Response) -> &str {
        response.headers()[header::LOCATION].to_str().unwrap()
    }

    /// A sign-in started: the login cookie, and the query of the redirect
    /// to the issuer.
    async fn start(app: &Router, issuer: &Issuer) -> (String, HashMap<String, String>) {
        let response = get_with(app, "/api/auth/mock/login", None).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let url = Url::parse(location(&response)).unwrap();
        assert_eq!(
            url.as_str().split('?').next(),
            Some(format!("{}/authorize", issuer.url).as_str())
        );
        let query = url.query_pairs().into_owned().collect();
        let cookies = set_cookies(&response);
        assert_eq!(cookies.len(), 1);
        assert!(cookies[0].starts_with("palimpseste_login="));
        (cookies[0].clone(), query)
    }

    /// Ends a sign-in: where it redirects to, and the session cookie set.
    async fn finish(
        app: &Router,
        login: Option<&str>,
        code: &str,
        state: &str,
    ) -> (String, Option<String>) {
        let uri = format!("/api/auth/mock/callback?code={code}&state={state}");
        let response = get_with(app, &uri, login).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let session = set_cookies(&response)
            .into_iter()
            .find(|cookie| cookie.starts_with("palimpseste_session="));
        (location(&response).to_owned(), session)
    }

    async fn me(app: &Router, session: &str) -> Value {
        let response = get_with(app, "/api/me", Some(session)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn signs_in_with_the_code_flow_and_pkce() {
        let issuer = Issuer::start().await;
        let app = app(&issuer);
        let (login, query) = start(&app, &issuer).await;
        assert_eq!(query["response_type"], "code");
        assert_eq!(query["client_id"], CLIENT_ID);
        assert_eq!(
            query["redirect_uri"],
            "http://localhost:3000/api/auth/mock/callback"
        );
        assert_eq!(query["code_challenge_method"], "S256");
        assert_eq!(query["scope"], "openid profile email");

        issuer.grant("one", "ana", &query["nonce"], &query["code_challenge"]);
        let (to, session) = finish(&app, Some(&login), "one", &query["state"]).await;
        assert_eq!(to, "/");
        let session = session.unwrap();
        assert_eq!(
            me(&app, &session).await,
            json!({"id": 1, "name": "ana Mock"})
        );

        // A callback is used once.
        issuer.grant("one", "ana", &query["nonce"], &query["code_challenge"]);
        let (to, replayed) = finish(&app, Some(&login), "one", &query["state"]).await;
        assert_eq!(to, "/login?signin=failed");
        assert_eq!(replayed, None);

        // The same identity is the same user; another one is another user.
        for (subject, id) in [("ana", 1), ("ben", 2)] {
            let (login, query) = start(&app, &issuer).await;
            issuer.grant("two", subject, &query["nonce"], &query["code_challenge"]);
            let (_, session) = finish(&app, Some(&login), "two", &query["state"]).await;
            assert_eq!(me(&app, &session.unwrap()).await["id"], id);
        }
    }

    #[tokio::test]
    async fn asks_for_the_scopes_of_the_provider() {
        let issuer = Issuer::start().await;
        let app = app_with(provider(&issuer).with_scopes(" identify  openid email "));
        let (login, query) = start(&app, &issuer).await;
        assert_eq!(query["scope"], "openid identify email");
        issuer.grant("one", "ana", &query["nonce"], &query["code_challenge"]);
        let (to, session) = finish(&app, Some(&login), "one", &query["state"]).await;
        assert_eq!(to, "/");
        assert!(session.is_some());

        let app = app_with(provider(&issuer).with_scopes(""));
        let (_, query) = start(&app, &issuer).await;
        assert_eq!(query["scope"], "openid");
    }

    #[tokio::test]
    async fn rejects_callbacks_not_matching_the_sign_in() {
        let issuer = Issuer::start().await;
        let app = app(&issuer);
        for case in [
            "wrong state",
            "no login cookie",
            "wrong nonce",
            "wrong verifier",
        ] {
            let (login, query) = start(&app, &issuer).await;
            let (mut nonce, mut challenge, mut state) = (
                query["nonce"].as_str(),
                query["code_challenge"].as_str(),
                query["state"].as_str(),
            );
            let mut cookie = Some(login.as_str());
            match case {
                "wrong state" => state = "forged",
                "no login cookie" => cookie = None,
                "wrong nonce" => nonce = "replayed",
                _ => challenge = "another",
            }
            issuer.grant("code", "ana", nonce, challenge);
            let (to, session) = finish(&app, cookie, "code", state).await;
            assert_eq!(to, "/login?signin=failed", "{case}");
            assert_eq!(session, None, "{case}");
        }
    }

    #[tokio::test]
    async fn reads_the_cookie_among_others() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "a=1; palimpseste_session=abc; b=2".parse().unwrap(),
        );
        assert_eq!(cookie(&headers, SESSION_COOKIE), Some("abc"));
        assert_eq!(cookie(&headers, LOGIN_COOKIE), None);
    }

    #[test]
    fn rejects_invalid_configuration() {
        assert!(Provider::new("Google", "G", "https://accounts.google.com", "c", "s").is_err());
        assert!(Provider::new("google", "G", "not a url", "c", "s").is_err());
        let provider = || Provider::new("dex", "Dex", "https://dex.test", "c", "s").unwrap();
        assert!(Auth::new("http://localhost", vec![provider(), provider()]).is_err());
        assert!(Auth::new("not a url", vec![provider()]).is_err());
        let auth = Auth::new("https://palimpseste.test", vec![provider()]).unwrap();
        assert_eq!(auth.provider_ids(), ["dex"]);
        assert!(
            auth.set_cookie(SESSION_COOKIE, "t", "/", SESSION_TTL)
                .ends_with("; Secure")
        );
    }
}
