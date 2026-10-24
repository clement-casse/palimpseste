//! The routes under `/api/auth` (ADR BE-API), and the cookies they
//! set. `login` and `link` keep the sign-in in progress under a short-lived
//! cookie and redirect to the provider; `callback` ends it, then sends the
//! browser back to the app, told how it went. An identity no user has is
//! sent to create its account, its registration kept under another cookie
//! until `registration` completes or cancels it (ADR BE-FIRST-SIGN-IN). The
//! session token travels in an `HttpOnly`, `SameSite=Lax` cookie.

use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{AppendHeaders, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use super::{AppState, Rejection, UserView, session_user};
use crate::{
    ActionContext, Error, Linking, Nickname, PendingRegistration, Profile, ProviderAnswer,
    ProviderId, REGISTRATION_TTL, RegistrationToken, SESSION_TTL, SIGN_IN_TTL, SessionToken,
    SignInEnd, SignInId,
};

const SESSION_COOKIE: &str = "palimpseste_session";
const LOGIN_COOKIE: &str = "palimpseste_login";
const LOGIN_COOKIE_PATH: &str = "/api/auth";
const REGISTRATION_COOKIE: &str = "palimpseste_registration";
const REGISTRATION_COOKIE_PATH: &str = "/api/auth/registration";
/// Where a sign-in ends: the app, the account creation page for an
/// identity no user has, or the sign-in page when it failed.
const SIGNED_IN: &str = "/";
const REGISTER: &str = "/register";
const SIGN_IN: &str = "/login";
const SIGN_IN_FAILED: &str = "/login?signin=failed";
/// Where linking an identity ends: the profile page, told how
/// (ADR BE-IDENTITIES).
const LINKED: &str = "/profile?link=done";
const LINK_ALREADY_YOURS: &str = "/profile?link=already";
const LINK_TAKEN: &str = "/profile?link=taken";
const LINK_PROVIDER_LINKED: &str = "/profile?link=provider";
const LINK_FAILED: &str = "/profile?link=failed";

/// The routes under `/api/auth`.
pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/providers", get(providers))
        .route("/{provider}/login", get(login))
        .route("/{provider}/link", get(link))
        .route("/{provider}/callback", get(callback))
        .route(
            "/registration",
            get(registration).post(register).delete(cancel_registration),
        )
        .route("/logout", post(logout))
}

/// How cookies are set: `Secure` when users reach the server over HTTPS.
#[derive(Debug, Clone, Copy)]
pub(super) struct Cookies {
    secure: bool,
}

impl Cookies {
    pub(super) const fn new(secure: bool) -> Self {
        Self { secure }
    }

    /// A `Set-Cookie` value; a zero `max_age` removes the cookie.
    fn set(self, name: &str, value: &str, path: &str, max_age: Duration) -> String {
        let secure = if self.secure { "; Secure" } else { "" };
        format!(
            "{name}={value}; Path={path}; Max-Age={}; HttpOnly; SameSite=Lax{secure}",
            max_age.as_secs()
        )
    }
}

/// The session token the request sends, if any.
pub(super) fn session_token(headers: &HeaderMap) -> Option<SessionToken> {
    cookie(headers, SESSION_COOKIE).map(SessionToken::from_cookie)
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

#[derive(Debug, Serialize)]
struct ProviderView {
    id: String,
    name: String,
}

async fn providers(State(state): State<AppState>) -> Json<Vec<ProviderView>> {
    Json(
        state
            .service
            .providers()
            .into_iter()
            .map(|provider| ProviderView {
                id: provider.id.as_str().to_owned(),
                name: provider.name,
            })
            .collect(),
    )
}

/// Starts signing in with a provider.
async fn login(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, Rejection> {
    start(&state, id.into(), None).await
}

/// Starts linking an identity at a provider to the signed-in user
/// (ADR BE-IDENTITIES). A signed-out browser is sent to sign in instead.
async fn link(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, Rejection> {
    match session_user(&state, &headers).await {
        Ok(user) => start(&state, id.into(), Some(ActionContext { user: user.id })).await,
        Err(Error::NoSession) => Ok(Redirect::to(SIGN_IN).into_response()),
        Err(error) => Err(error.into()),
    }
}

/// Redirects to provider `id`, keeping the sign-in in progress under a
/// cookie of this browser.
async fn start(
    state: &AppState,
    id: ProviderId,
    link: Option<ActionContext>,
) -> Result<Response, Rejection> {
    match state.service.start_sign_in(&id, link).await {
        Ok(started) => {
            let cookie = state.cookies.set(
                LOGIN_COOKIE,
                started.id.as_str(),
                LOGIN_COOKIE_PATH,
                SIGN_IN_TTL,
            );
            Ok(([(header::SET_COOKIE, cookie)], Redirect::to(&started.url)).into_response())
        }
        Err(Error::SignInFailed { linking, cause }) => {
            tracing::warn!(provider = %id, reason = format!("{cause:#}"), linking, "sign-in failed");
            Ok(Redirect::to(failed(linking)).into_response())
        }
        Err(error) => Err(error.into()),
    }
}

/// Where a failed sign-in, or a failed link, sends the browser.
const fn failed(linking: bool) -> &'static str {
    if linking { LINK_FAILED } else { SIGN_IN_FAILED }
}

/// What a provider sends back to the callback.
#[derive(Debug, Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// Ends signing in, or linking, with a provider: opens a session or links
/// the identity, or tells the app it failed. The login cookie is removed
/// either way.
async fn callback(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(callback): Query<Callback>,
    headers: HeaderMap,
) -> Response {
    let clear_login = state
        .cookies
        .set(LOGIN_COOKIE, "", LOGIN_COOKIE_PATH, Duration::ZERO);
    let id = ProviderId::from(id);
    let sign_in = cookie(&headers, LOGIN_COOKIE).map(SignInId::from_cookie);
    let answer = ProviderAnswer {
        code: callback.code,
        state: callback.state,
        error: callback.error,
    };
    let to = match state.service.finish_sign_in(&id, sign_in, answer).await {
        Ok(SignInEnd::SignedIn(token)) => {
            let session = state
                .cookies
                .set(SESSION_COOKIE, token.as_str(), "/", SESSION_TTL);
            return (
                AppendHeaders([
                    (header::SET_COOKIE, clear_login),
                    (header::SET_COOKIE, session),
                ]),
                Redirect::to(SIGNED_IN),
            )
                .into_response();
        }
        Ok(SignInEnd::Registering(token)) => {
            let registration = state.cookies.set(
                REGISTRATION_COOKIE,
                token.as_str(),
                REGISTRATION_COOKIE_PATH,
                REGISTRATION_TTL,
            );
            return (
                AppendHeaders([
                    (header::SET_COOKIE, clear_login),
                    (header::SET_COOKIE, registration),
                ]),
                Redirect::to(REGISTER),
            )
                .into_response();
        }
        Ok(SignInEnd::Linked(Linking::Linked)) => LINKED,
        Ok(SignInEnd::Linked(Linking::AlreadyYours)) => LINK_ALREADY_YOURS,
        Err(Error::IdentityTaken) => LINK_TAKEN,
        Err(Error::ProviderAlreadyLinked) => LINK_PROVIDER_LINKED,
        Err(Error::SignInFailed { linking, cause }) => {
            tracing::warn!(provider = %id, reason = format!("{cause:#}"), linking, "sign-in failed");
            failed(linking)
        }
        Err(error) => {
            tracing::error!(provider = %id, error = format!("{error:#}"), "sign-in failed");
            SIGN_IN_FAILED
        }
    };
    ([(header::SET_COOKIE, clear_login)], Redirect::to(to)).into_response()
}

/// The registration in progress of the request; [`Error::NoRegistration`]
/// without its cookie.
fn registration_token(headers: &HeaderMap) -> Result<RegistrationToken, Error> {
    cookie(headers, REGISTRATION_COOKIE)
        .map(RegistrationToken::from_cookie)
        .ok_or(Error::NoRegistration)
}

/// A registration in progress, as the person creating their account sees
/// it.
#[derive(Debug, Serialize)]
struct RegistrationView {
    /// The nickname suggested.
    nickname: String,
    /// The name of the identity's provider, or its issuer.
    provider: String,
    label: String,
}

/// The account a registration creates.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NewAccount {
    nickname: String,
    #[serde(default)]
    profile: String,
}

/// The registration in progress in this browser.
async fn registration(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RegistrationView>, Rejection> {
    let token = registration_token(&headers)?;
    let PendingRegistration {
        registration,
        provider,
    } = state.service.registration(&token).await?;
    Ok(Json(RegistrationView {
        nickname: registration.nickname.0,
        provider: provider.map_or(registration.issuer, |provider| provider.name),
        label: registration.label,
    }))
}

/// Completes the registration in progress: creates the account and signs
/// it in.
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(account): Json<NewAccount>,
) -> Result<Response, Rejection> {
    let token = registration_token(&headers)?;
    let nickname = Nickname::parse(&account.nickname)?;
    let profile = Profile::parse(&account.profile)?;
    let (user, session) = state.service.register(&token, nickname, profile).await?;
    let clear = state.cookies.set(
        REGISTRATION_COOKIE,
        "",
        REGISTRATION_COOKIE_PATH,
        Duration::ZERO,
    );
    let session = state
        .cookies
        .set(SESSION_COOKIE, session.as_str(), "/", SESSION_TTL);
    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::SET_COOKIE, clear), (header::SET_COOKIE, session)]),
        Json(UserView::from(user)),
    )
        .into_response())
}

/// Cancels the registration in progress, if any, and removes its cookie:
/// the identity stays free.
async fn cancel_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Rejection> {
    if let Ok(token) = registration_token(&headers) {
        state.service.cancel_registration(&token).await?;
    }
    let clear = state.cookies.set(
        REGISTRATION_COOKIE,
        "",
        REGISTRATION_COOKIE_PATH,
        Duration::ZERO,
    );
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, clear)]))
}

/// Ends the session of the request, if any, and removes its cookie.
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Rejection> {
    if let Some(token) = session_token(&headers) {
        state.service.sign_out(&token).await?;
    }
    let clear = state.cookies.set(SESSION_COOKIE, "", "/", Duration::ZERO);
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, clear)]))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::{SystemTime, UNIX_EPOCH},
    };

    use axum::{
        Form,
        body::Body,
        http::{Method, Request},
    };
    use openidconnect::{
        JsonWebKeyId, PkceCodeChallenge, PkceCodeVerifier, PrivateSigningKey,
        core::{CoreIdToken, CoreIdTokenClaims, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey},
        url::Url,
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::{HttpApi, OidcProviders, PalimpsesteServiceImpl, ProviderConfig, SqliteRepo};

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
                    include_str!("../../../tests/fixtures/issuer-key.pem"),
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
                        "email": format!("{}@mail.test", grant.subject),
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

    fn provider(issuer: &Issuer) -> ProviderConfig {
        ProviderConfig::new("mock", "Mock", &issuer.url, CLIENT_ID, "secret").unwrap()
    }

    fn app(issuer: &Issuer) -> Router {
        app_with(vec![provider(issuer)])
    }

    fn app_with(providers: Vec<ProviderConfig>) -> Router {
        let public_url = "http://localhost:3000/";
        let providers = OidcProviders::new(public_url, providers).unwrap();
        let service = Arc::new(PalimpsesteServiceImpl::new(
            SqliteRepo::in_memory(),
            providers,
        ));
        let dir = std::env::temp_dir().join("palimpseste-backend-auth");
        HttpApi::new(service, &dir, public_url).router()
    }

    async fn send(
        app: &Router,
        method: Method,
        uri: &str,
        cookie: Option<&str>,
        body: Option<Value>,
    ) -> Response {
        let request = cookie.into_iter().fold(
            Request::builder().method(method).uri(uri),
            |request, cookie| request.header(header::COOKIE, cookie),
        );
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .unwrap();
        app.clone().oneshot(request).await.unwrap()
    }

    async fn get_with(app: &Router, uri: &str, cookie: Option<&str>) -> Response {
        send(app, Method::GET, uri, cookie, None).await
    }

    async fn body_json(response: Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
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

    /// The `name=value` of cookie `name`, if the response sets it.
    fn set_cookie(response: &Response, name: &str) -> Option<String> {
        set_cookies(response)
            .into_iter()
            .find(|cookie| cookie.starts_with(&format!("{name}=")))
    }

    fn location(response: &Response) -> &str {
        response.headers()[header::LOCATION].to_str().unwrap()
    }

    /// A sign-in started: the login cookie, and the query of the redirect
    /// to the issuer.
    async fn start(app: &Router, issuer: &Issuer) -> (String, HashMap<String, String>) {
        start_at(app, issuer, "/api/auth/mock/login", None).await
    }

    async fn start_at(
        app: &Router,
        issuer: &Issuer,
        uri: &str,
        session: Option<&str>,
    ) -> (String, HashMap<String, String>) {
        let response = get_with(app, uri, session).await;
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

    /// How a callback ended: where it redirects to, and the session and
    /// registration cookies it sets.
    #[derive(Debug)]
    struct Ended {
        to: String,
        session: Option<String>,
        registration: Option<String>,
    }

    /// Ends a sign-in at the mock provider.
    async fn finish(app: &Router, login: Option<&str>, code: &str, state: &str) -> Ended {
        finish_at(app, "mock", login, code, state).await
    }

    async fn finish_at(
        app: &Router,
        provider: &str,
        login: Option<&str>,
        code: &str,
        state: &str,
    ) -> Ended {
        let uri = format!("/api/auth/{provider}/callback?code={code}&state={state}");
        let response = get_with(app, &uri, login).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        Ended {
            to: location(&response).to_owned(),
            session: set_cookie(&response, SESSION_COOKIE),
            registration: set_cookie(&response, REGISTRATION_COOKIE),
        }
    }

    async fn me(app: &Router, session: &str) -> Value {
        let response = get_with(app, "/api/me", Some(session)).await;
        assert_eq!(response.status(), StatusCode::OK);
        body_json(response).await
    }

    /// Signs in as `subject` at the mock provider, to the end of the
    /// callback.
    async fn sign_in_as(app: &Router, issuer: &Issuer, subject: &str) -> Ended {
        sign_in_at(app, issuer, "mock", subject).await
    }

    async fn sign_in_at(app: &Router, issuer: &Issuer, provider: &str, subject: &str) -> Ended {
        let uri = format!("/api/auth/{provider}/login");
        let (login, query) = start_at(app, issuer, &uri, None).await;
        issuer.grant("in", subject, &query["nonce"], &query["code_challenge"]);
        finish_at(app, provider, Some(&login), "in", &query["state"]).await
    }

    /// Signs in as `subject`, a registered identity; gives the session
    /// cookie.
    async fn sign_in(app: &Router, issuer: &Issuer, subject: &str) -> String {
        let ended = sign_in_as(app, issuer, subject).await;
        assert_eq!(ended.to, "/");
        assert_eq!(ended.registration, None);
        ended.session.unwrap()
    }

    /// Signs in as `subject` for the first time; gives the registration
    /// cookie.
    async fn start_registration(app: &Router, issuer: &Issuer, subject: &str) -> String {
        let ended = sign_in_as(app, issuer, subject).await;
        assert_eq!(ended.to, "/register");
        assert_eq!(ended.session, None, "a registration opens no session");
        ended.registration.unwrap()
    }

    const REGISTRATION: &str = "/api/auth/registration";

    /// Completes the registration of the browser with `account`.
    async fn register(app: &Router, registration: Option<&str>, account: Value) -> Response {
        send(app, Method::POST, REGISTRATION, registration, Some(account)).await
    }

    /// Registers `subject` with the nickname suggested; gives the session
    /// cookie.
    async fn sign_up(app: &Router, issuer: &Issuer, subject: &str) -> String {
        let registration = start_registration(app, issuer, subject).await;
        let suggested = body_json(get_with(app, REGISTRATION, Some(&registration)).await).await;
        let response = register(
            app,
            Some(&registration),
            json!({"nickname": suggested["nickname"]}),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        set_cookie(&response, SESSION_COOKIE).unwrap()
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
        let ended = finish(&app, Some(&login), "one", &query["state"]).await;
        assert_eq!(ended.to, "/register");
        assert!(ended.registration.is_some());

        // A callback is used once.
        issuer.grant("one", "ana", &query["nonce"], &query["code_challenge"]);
        let replayed = finish(&app, Some(&login), "one", &query["state"]).await;
        assert_eq!(replayed.to, "/login?signin=failed");
        assert_eq!(replayed.session, None);
        assert_eq!(replayed.registration, None);
    }

    #[tokio::test]
    async fn the_first_sign_in_creates_the_account() {
        let issuer = Issuer::start().await;
        let app = app(&issuer);
        let registration = start_registration(&app, &issuer, "ana").await;
        let response = get_with(&app, "/api/me", Some(&registration)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let response = get_with(&app, REGISTRATION, Some(&registration)).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_json(response).await,
            json!({"nickname": "ana Mock", "provider": "Mock", "label": "ana@mail.test"})
        );

        for (account, status) in [
            (json!({"nickname": " "}), StatusCode::UNPROCESSABLE_ENTITY),
            (
                json!({"nickname": "Ana", "id": 1}),
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
        ] {
            let response = register(&app, Some(&registration), account.clone()).await;
            assert_eq!(response.status(), status, "{account}");
        }
        let response = register(&app, None, json!({"nickname": "Ana"})).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = register(
            &app,
            Some(&registration),
            json!({"nickname": " Anaïs ", "profile": " Joue une barbare. "}),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            set_cookie(&response, REGISTRATION_COOKIE).as_deref(),
            Some("palimpseste_registration=")
        );
        let session = set_cookie(&response, SESSION_COOKIE).unwrap();
        let created = json!({"nickname": "Anaïs", "profile": "Joue une barbare."});
        assert_eq!(body_json(response).await, created);
        assert_eq!(me(&app, &session).await, created);

        // The registration is used up; the next sign-in goes straight in.
        let response = register(&app, Some(&registration), json!({"nickname": "Ana"})).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            me(&app, &sign_in(&app, &issuer, "ana").await).await,
            created
        );

        // Another identity is another account, its nickname its own.
        let ben = start_registration(&app, &issuer, "ben").await;
        let response = register(&app, Some(&ben), json!({"nickname": "anaïs"})).await;
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let response = register(&app, Some(&ben), json!({"nickname": "Ben"})).await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn a_cancelled_registration_leaves_the_identity_free() {
        let issuer = Issuer::start().await;
        let app = app(&issuer);
        let registration = start_registration(&app, &issuer, "ana").await;
        let response = send(
            &app,
            Method::DELETE,
            REGISTRATION,
            Some(&registration),
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let cleared = response.headers()[header::SET_COOKIE].to_str().unwrap();
        assert!(
            cleared
                .starts_with("palimpseste_registration=; Path=/api/auth/registration; Max-Age=0"),
            "{cleared}"
        );
        let response = get_with(&app, REGISTRATION, Some(&registration)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let response = register(&app, Some(&registration), json!({"nickname": "Ana"})).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let response = send(&app, Method::DELETE, REGISTRATION, None, None).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Signing in again starts another registration.
        start_registration(&app, &issuer, "ana").await;
    }

    /// Links `subject` at `provider` of `issuer` to the user of `session`;
    /// gives where it ends.
    async fn link(
        app: &Router,
        issuer: &Issuer,
        provider: &str,
        session: &str,
        subject: &str,
    ) -> String {
        let uri = format!("/api/auth/{provider}/link");
        let (login, query) = start_at(app, issuer, &uri, Some(session)).await;
        issuer.grant("link", subject, &query["nonce"], &query["code_challenge"]);
        let ended = finish_at(app, provider, Some(&login), "link", &query["state"]).await;
        assert_eq!(ended.session, None, "linking opens no session");
        assert_eq!(ended.registration, None, "linking registers nobody");
        ended.to
    }

    async fn identities(app: &Router, session: &str) -> Value {
        let response = get_with(app, "/api/me/identities", Some(session)).await;
        assert_eq!(response.status(), StatusCode::OK);
        body_json(response).await
    }

    #[tokio::test]
    async fn links_one_identity_per_provider_to_the_signed_in_user() {
        let (issuer, other) = (Issuer::start().await, Issuer::start().await);
        let app = app_with(vec![
            provider(&issuer),
            ProviderConfig::new("other", "Other", &other.url, CLIENT_ID, "secret").unwrap(),
        ]);
        let ana = sign_up(&app, &issuer, "ana").await;
        let ben = sign_up(&app, &issuer, "ben").await;

        assert_eq!(
            link(&app, &other, "other", &ana, "ana-2").await,
            "/profile?link=done"
        );
        assert_eq!(
            identities(&app, &ana).await,
            json!([
                {"provider": "mock", "name": "Mock", "label": "ana@mail.test"},
                {"provider": "other", "name": "Other", "label": "ana-2@mail.test"},
            ])
        );
        // The linked identity signs in as Ana.
        let again = sign_in_at(&app, &other, "other", "ana-2").await;
        assert_eq!(again.to, "/");
        assert_eq!(
            me(&app, &again.session.unwrap()).await["nickname"],
            "ana Mock"
        );

        assert_eq!(
            link(&app, &other, "other", &ana, "ana-2").await,
            "/profile?link=already"
        );
        assert_eq!(
            link(&app, &issuer, "mock", &ana, "ben").await,
            "/profile?link=taken"
        );
        assert_eq!(
            link(&app, &issuer, "mock", &ana, "ana-3").await,
            "/profile?link=provider"
        );
        assert_eq!(identities(&app, &ben).await.as_array().unwrap().len(), 1);
        assert_eq!(identities(&app, &ana).await.as_array().unwrap().len(), 2);

        // A link checks the callback as a sign-in does.
        let (login, query) = start_at(&app, &other, "/api/auth/other/link", Some(&ben)).await;
        other.grant("forged", "ben-2", &query["nonce"], &query["code_challenge"]);
        let ended = finish_at(&app, "other", Some(&login), "forged", "forged").await;
        assert_eq!(ended.to, "/profile?link=failed");
        assert_eq!(identities(&app, &ben).await.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn linking_needs_a_session() {
        let issuer = Issuer::start().await;
        let app = app(&issuer);
        for session in [None, Some("palimpseste_session=unknown")] {
            let response = get_with(&app, "/api/auth/mock/link", session).await;
            assert_eq!(response.status(), StatusCode::SEE_OTHER);
            assert_eq!(location(&response), "/login");
            assert_eq!(set_cookies(&response), Vec::<String>::new());
        }
    }

    #[tokio::test]
    async fn asks_for_the_scopes_of_the_provider() {
        let issuer = Issuer::start().await;
        let app = app_with(vec![
            provider(&issuer).with_scopes(" identify  openid email "),
        ]);
        let (login, query) = start(&app, &issuer).await;
        assert_eq!(query["scope"], "openid identify email");
        issuer.grant("one", "ana", &query["nonce"], &query["code_challenge"]);
        let ended = finish(&app, Some(&login), "one", &query["state"]).await;
        assert_eq!(ended.to, "/register");

        let app = app_with(vec![provider(&issuer).with_scopes("")]);
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
            let ended = finish(&app, cookie, "code", state).await;
            assert_eq!(ended.to, "/login?signin=failed", "{case}");
            assert_eq!(ended.session, None, "{case}");
            assert_eq!(ended.registration, None, "{case}");
        }
    }

    #[test]
    fn reads_the_cookie_among_others() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "a=1; palimpseste_session=abc; b=2".parse().unwrap(),
        );
        assert_eq!(cookie(&headers, SESSION_COOKIE), Some("abc"));
        assert_eq!(cookie(&headers, LOGIN_COOKIE), None);
    }

    #[test]
    fn cookies_are_secure_over_https() {
        let session = |secure| Cookies::new(secure).set(SESSION_COOKIE, "t", "/", SESSION_TTL);
        assert!(session(true).ends_with("; Secure"));
        assert!(!session(false).contains("Secure"));
    }

    #[tokio::test]
    async fn technical_failures_of_a_sign_in_are_not_detailed() {
        let down = || ProviderId::from("down");
        let mut service = crate::MockPalimpsesteService::new();
        service
            .expect_session_user()
            .returning(|_| Err(anyhow::anyhow!("the database is gone").into()));
        service
            .expect_start_sign_in()
            .withf(move |provider, _| *provider == down())
            .returning(|_, link| {
                Err(Error::SignInFailed {
                    linking: link.is_some(),
                    cause: anyhow::anyhow!("discovery failed"),
                })
            });
        service
            .expect_start_sign_in()
            .withf(move |provider, _| *provider != down())
            .returning(|_, _| Err(anyhow::anyhow!("the database is gone").into()));
        service
            .expect_finish_sign_in()
            .returning(|_, _, _| Err(anyhow::anyhow!("the database is gone").into()));
        let dir = std::env::temp_dir().join("palimpseste-backend-auth");
        let app = HttpApi::new(Arc::new(service), &dir, "http://localhost:3000/").router();

        let link = get_with(&app, "/api/auth/mock/link", Some("palimpseste_session=t")).await;
        assert_eq!(link.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let login = get_with(&app, "/api/auth/mock/login", None).await;
        assert_eq!(login.status(), StatusCode::INTERNAL_SERVER_ERROR);
        // An unreachable provider sends back to the sign-in page, told so.
        let login = get_with(&app, "/api/auth/down/login", None).await;
        assert_eq!(location(&login), SIGN_IN_FAILED);
        let callback = get_with(&app, "/api/auth/mock/callback?code=c&state=s", None).await;
        assert_eq!(location(&callback), SIGN_IN_FAILED);
    }
}
