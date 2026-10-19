//! The HTTP adapter: the JSON API under `/api`, the health check at
//! `/healthz`, the front end everywhere else.
//!
//! The API serves, checks and stores (ADR BE-CHECKS): it rejects a request
//! it cannot read, never a character breaking a rule. Signing in is under
//! `/api/auth` (ADR BE-SIGN-IN); every other route needs a session and
//! reaches the signed-in user's profile, identities (ADR BE-IDENTITIES),
//! characters, invitations and friends only, and reads their friends'
//! characters (ADR BE-FRIENDS). Each request is logged with its status and latency,
//! each rejected request with why (ADR BE-LOGS).

mod friends;
mod frontend;
mod sign_in;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path as UrlPath, State},
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
    routing::{any, get, put},
};
use palimpseste_document::{Document, Engine};
use serde::{Deserialize, Serialize};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use self::sign_in::Cookies;
use crate::{
    ActionContext, Character, CharacterId, CharacterName, Error, LinkedIdentity, Nickname,
    PalimpsesteService, Profile, User, Version,
};

/// The HTTP server of the use cases: the router of the whole application.
pub struct HttpApi {
    service: Arc<dyn PalimpsesteService>,
    frontend_dir: PathBuf,
    cookies: Cookies,
}

impl HttpApi {
    /// Serves `service`, and the built front end from `frontend_dir`, to
    /// users reaching the server at `public_url`: cookies are `Secure` when
    /// it is `https`.
    pub fn new(
        service: Arc<dyn PalimpsesteService>,
        frontend_dir: &Path,
        public_url: &str,
    ) -> Self {
        Self {
            service,
            frontend_dir: frontend_dir.to_owned(),
            cookies: Cookies::new(public_url.starts_with("https://")),
        }
    }

    /// The router. `/healthz` is the health check; other paths outside
    /// `/api` are the front end's. Unknown paths under `/api` are not found.
    /// Every request but the health check is traced.
    pub fn router(&self) -> Router {
        let not_found = || async { StatusCode::NOT_FOUND };
        let state = AppState {
            service: Arc::clone(&self.service),
            cookies: self.cookies,
        };
        let api = Router::new()
            .nest("/auth", sign_in::routes())
            .route("/me", get(me).patch(update_me))
            .route("/me/identities", get(identities))
            .route(
                "/me/characters",
                get(list_characters).post(create_character),
            )
            .route(
                "/characters/{character_id}",
                get(get_character)
                    .patch(rename_character)
                    .delete(delete_character),
            )
            .route("/characters/{character_id}/document", put(put_document))
            .merge(friends::routes())
            .fallback(not_found)
            .with_state(state.clone());
        Router::new()
            .nest("/api", api)
            // `nest` leaves the bare trailing slash to the front end.
            .route("/api/", any(not_found))
            .fallback_service(frontend::router(&self.frontend_dir))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            // After the trace layer: probed every few seconds, it would
            // flood the logs.
            .route("/healthz", get(healthz).with_state(state))
    }
}

/// What the handlers share.
#[derive(Clone)]
struct AppState {
    service: Arc<dyn PalimpsesteService>,
    cookies: Cookies,
}

/// Why a request failed, besides the JSON the extractors reject (`400` for
/// malformed JSON, `422` for an unknown shape).
#[derive(Debug)]
enum Rejection {
    /// A use case failed.
    Service(Error),
    /// A document replaced without `If-Match`.
    PreconditionRequired,
}

impl From<Error> for Rejection {
    fn from(error: Error) -> Self {
        Self::Service(error)
    }
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        let error = match self {
            Self::PreconditionRequired => {
                tracing::warn!("document replaced without If-Match");
                return (StatusCode::PRECONDITION_REQUIRED, "If-Match is required").into_response();
            }
            Self::Service(error) => error,
        };
        match error {
            Error::NoSession => StatusCode::UNAUTHORIZED.into_response(),
            Error::UnknownProvider
            | Error::CharacterNotFound
            | Error::NoRegistration
            | Error::InvitationNotFound
            | Error::FriendNotFound => StatusCode::NOT_FOUND.into_response(),
            Error::BlankNickname
            | Error::NicknameTooLong
            | Error::ProfileTooLong
            | Error::BlankName
            | Error::UnreadableDocument(_)
            | Error::OtherEngine
            | Error::OwnInvitation => {
                let reason = error.to_string();
                tracing::warn!(%reason, "unprocessable request");
                (StatusCode::UNPROCESSABLE_ENTITY, reason).into_response()
            }
            Error::NicknameTaken
            | Error::IdentityTaken
            | Error::ProviderAlreadyLinked
            | Error::AlreadyFriends => {
                let reason = error.to_string();
                tracing::info!(%reason, "conflicting request");
                (StatusCode::CONFLICT, reason).into_response()
            }
            Error::Stale(version) => {
                tracing::info!(current = %version, "document replaced at a stale version");
                (
                    StatusCode::PRECONDITION_FAILED,
                    [(header::ETAG, etag(version))],
                )
                    .into_response()
            }
            Error::SignInFailed { cause, .. } | Error::Underlying(cause) => {
                tracing::error!(error = format!("{cause:#}"), "internal error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

/// The signed-in user of a request: `401` without a live session.
#[derive(Debug)]
struct SignedIn(User);

impl SignedIn {
    const fn context(&self) -> ActionContext {
        ActionContext { user: self.0.id }
    }
}

impl FromRequestParts<AppState> for SignedIn {
    type Rejection = Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Rejection> {
        Ok(Self(session_user(state, &parts.headers).await?))
    }
}

/// The user of the session the request sends, unless it is unknown or
/// expired.
async fn session_user(state: &AppState, headers: &HeaderMap) -> Result<User, Error> {
    let token = sign_in::session_token(headers).ok_or(Error::NoSession)?;
    state.service.session_user(&token).await
}

/// A user, as they see themselves: never their id.
#[derive(Debug, Serialize)]
struct UserView {
    nickname: String,
    profile: String,
}

impl From<User> for UserView {
    fn from(user: User) -> Self {
        Self {
            nickname: user.nickname.0,
            profile: user.profile.0,
        }
    }
}

/// An identity of the signed-in user, named by its provider.
#[derive(Debug, Serialize)]
struct IdentityView {
    /// The id of its provider, unless that provider is no longer
    /// configured.
    provider: Option<String>,
    /// The name of its provider, or its issuer.
    name: String,
    label: String,
}

impl From<LinkedIdentity> for IdentityView {
    fn from(LinkedIdentity { identity, provider }: LinkedIdentity) -> Self {
        Self {
            provider: provider
                .as_ref()
                .map(|provider| provider.id.as_str().to_owned()),
            name: provider.map_or(identity.issuer, |provider| provider.name),
            label: identity.label,
        }
    }
}

#[derive(Debug, Serialize)]
struct CharacterView {
    id: i64,
    name: String,
    engine: Engine,
    document: Document,
    version: i64,
}

impl From<Character> for CharacterView {
    fn from(character: Character) -> Self {
        Self {
            id: character.id.get(),
            name: character.name.0,
            engine: character.engine,
            document: character.document,
            version: character.version.get(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct NewCharacter {
    name: String,
    engine: Engine,
}

#[derive(Debug, Deserialize)]
struct Rename {
    name: String,
}

/// A change to the signed-in user: the fields given change.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileChange {
    nickname: Option<String>,
    profile: Option<String>,
}

/// The `ETag` of a document version.
fn etag(version: Version) -> String {
    format!("\"{version}\"")
}

/// The version `If-Match` expects. A value naming no version gives version
/// 0, which is never stored, so never matches.
fn if_match(headers: &HeaderMap) -> Result<Version, Rejection> {
    let value = headers
        .get(header::IF_MATCH)
        .ok_or(Rejection::PreconditionRequired)?;
    let version = value
        .to_str()
        .ok()
        .and_then(|value| value.trim().strip_prefix('"')?.strip_suffix('"'))
        .and_then(|version| version.parse().ok())
        .unwrap_or(0);
    Ok(Version::from(version))
}

/// The health check of the hosting platform: `200 ok` while the server can
/// serve, else `503`, never cached.
async fn healthz(State(state): State<AppState>) -> Response {
    let no_store = [(header::CACHE_CONTROL, "no-store")];
    match state.service.health().await {
        Ok(()) => (no_store, "ok").into_response(),
        Err(error) => {
            tracing::error!(error = format!("{error:#}"), "unhealthy");
            (StatusCode::SERVICE_UNAVAILABLE, no_store).into_response()
        }
    }
}

async fn me(SignedIn(user): SignedIn) -> Json<UserView> {
    Json(user.into())
}

async fn update_me(
    signed_in: SignedIn,
    State(state): State<AppState>,
    Json(change): Json<ProfileChange>,
) -> Result<Json<UserView>, Rejection> {
    let nickname = change
        .nickname
        .as_deref()
        .map(Nickname::parse)
        .transpose()?;
    let profile = change.profile.as_deref().map(Profile::parse).transpose()?;
    let user = state
        .service
        .update_profile(&signed_in.context(), nickname, profile)
        .await?;
    Ok(Json(user.into()))
}

async fn identities(
    signed_in: SignedIn,
    State(state): State<AppState>,
) -> Result<Json<Vec<IdentityView>>, Rejection> {
    let identities = state.service.identities(&signed_in.context()).await?;
    Ok(Json(identities.into_iter().map(Into::into).collect()))
}

async fn list_characters(
    signed_in: SignedIn,
    State(state): State<AppState>,
) -> Result<Json<Vec<CharacterView>>, Rejection> {
    let characters = state.service.characters(&signed_in.context()).await?;
    Ok(Json(characters.into_iter().map(Into::into).collect()))
}

async fn create_character(
    signed_in: SignedIn,
    State(state): State<AppState>,
    Json(new): Json<NewCharacter>,
) -> Result<(StatusCode, Json<CharacterView>), Rejection> {
    let name = CharacterName::parse(&new.name)?;
    let character = state
        .service
        .create_character(&signed_in.context(), name, new.engine)
        .await?;
    Ok((StatusCode::CREATED, Json(character.into())))
}

async fn get_character(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
) -> Result<impl IntoResponse, Rejection> {
    let character = state
        .service
        .character(&signed_in.context(), CharacterId::from(id))
        .await?;
    Ok((
        [(header::ETAG, etag(character.version))],
        Json(CharacterView::from(character)),
    ))
}

async fn rename_character(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
    Json(rename): Json<Rename>,
) -> Result<Json<CharacterView>, Rejection> {
    let name = CharacterName::parse(&rename.name)?;
    let character = state
        .service
        .rename_character(&signed_in.context(), CharacterId::from(id), name)
        .await?;
    Ok(Json(character.into()))
}

async fn delete_character(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
) -> Result<StatusCode, Rejection> {
    state
        .service
        .delete_character(&signed_in.context(), CharacterId::from(id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Replaces a character's whole document, at the version `If-Match` names.
async fn put_document(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
    headers: HeaderMap,
    Json(document): Json<Document>,
) -> Result<impl IntoResponse, Rejection> {
    let expected = if_match(&headers)?;
    let version = state
        .service
        .replace_document(
            &signed_in.context(),
            CharacterId::from(id),
            expected,
            document,
        )
        .await?;
    Ok((StatusCode::NO_CONTENT, [(header::ETAG, etag(version))]))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use anyhow::anyhow;
    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, Method, Request},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::{
        MockPalimpsesteService, OidcProviders, PalimpsesteRepository, PalimpsesteServiceImpl,
        ProviderConfig, SESSION_TTL, SessionToken, SqliteRepo, UserId,
    };

    /// A frontend directory unique to the test, holding an `index.html`.
    pub(super) fn frontend_dir(test: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("palimpseste-backend-{}-{test}", std::process::id()));
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("index.html"), "<p>index</p>").unwrap();
        fs::write(dir.join("app.js"), "// app").unwrap();
        fs::write(dir.join("sw.js"), "// service worker").unwrap();
        fs::write(dir.join("manifest.webmanifest"), "{}").unwrap();
        fs::write(dir.join("assets/engine-abc123.wasm"), b"\0asm").unwrap();
        dir
    }

    /// Opens a session of `user` and gives the `Cookie` header that sends it.
    async fn session_cookie(repo: &SqliteRepo, user: UserId) -> String {
        let token = SessionToken::random();
        repo.create_session(user, &token.hash(), SESSION_TTL)
            .await
            .unwrap();
        format!("palimpseste_session={}", token.as_str())
    }

    /// The application, with users Ana (1), Ben (2) and Cy (3) signed in:
    /// the `Cookie` header of each.
    pub(super) struct App {
        pub(super) router: Router,
        pub(super) ana: String,
        pub(super) ben: String,
        pub(super) cy: String,
    }

    pub(super) async fn app(test: &str) -> App {
        let repo = SqliteRepo::in_memory();
        let mut cookies = Vec::new();
        for name in ["Ana", "Ben", "Cy"] {
            let token = SessionToken::random().hash();
            let label = format!("{name}@mail.test");
            repo.create_registration(
                &token,
                "https://issuer.test",
                name,
                name,
                &label,
                SESSION_TTL,
            )
            .await
            .unwrap();
            let user = repo
                .register(&token, Nickname::parse(name).unwrap(), Profile::default())
                .await
                .unwrap()
                .unwrap();
            cookies.push(session_cookie(&repo, user.id).await);
        }
        let provider =
            ProviderConfig::new("dex", "Dex", "https://issuer.test", "client", "secret").unwrap();
        let providers = OidcProviders::new("http://localhost:3000", vec![provider]).unwrap();
        let service = Arc::new(PalimpsesteServiceImpl::new(repo, providers));
        let cy = cookies.pop().unwrap();
        let ben = cookies.pop().unwrap();
        let ana = cookies.pop().unwrap();
        App {
            router: HttpApi::new(service, &frontend_dir(test), "http://localhost:3000").router(),
            ana,
            ben,
            cy,
        }
    }

    /// Sends a request with `cookie`, if any, and `headers`.
    pub(super) async fn send_with(
        router: &Router,
        cookie: Option<&str>,
        method: Method,
        uri: &str,
        headers: &[(header::HeaderName, &str)],
        body: Option<Value>,
    ) -> (StatusCode, HeaderMap, Vec<u8>) {
        let request = headers
            .iter()
            .chain(cookie.map(|c| (header::COOKIE, c)).iter())
            .fold(
                Request::builder().method(method).uri(uri),
                |request, (name, value)| request.header(name, *value),
            );
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, headers, bytes.to_vec())
    }

    impl App {
        /// Sends a request as Ana.
        async fn send_full(
            &self,
            method: Method,
            uri: &str,
            body: Option<Value>,
        ) -> (StatusCode, HeaderMap, Vec<u8>) {
            send_with(&self.router, Some(&self.ana), method, uri, &[], body).await
        }

        async fn send(
            &self,
            method: Method,
            uri: &str,
            body: Option<Value>,
        ) -> (StatusCode, Vec<u8>) {
            let (status, _, body) = self.send_full(method, uri, body).await;
            (status, body)
        }

        async fn send_json(
            &self,
            method: Method,
            uri: &str,
            body: Option<Value>,
        ) -> (StatusCode, Value) {
            let (status, bytes) = self.send(method, uri, body).await;
            (status, serde_json::from_slice(&bytes).unwrap())
        }

        /// Sends a request without a cookie.
        async fn anonymous(&self, method: Method, uri: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
            send_with(&self.router, None, method, uri, &[], None).await
        }

        async fn put_document(
            &self,
            version: Option<&str>,
            document: Value,
        ) -> (StatusCode, HeaderMap, Vec<u8>) {
            let headers: Vec<(header::HeaderName, &str)> =
                version.map(|v| (header::IF_MATCH, v)).into_iter().collect();
            send_with(
                &self.router,
                Some(&self.ana),
                Method::PUT,
                "/api/characters/1/document",
                &headers,
                Some(document),
            )
            .await
        }
    }

    #[tokio::test]
    async fn me_is_the_signed_in_user() {
        let app = app("me").await;
        let (status, ana) = app.send_json(Method::GET, "/api/me", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ana, json!({"nickname": "Ana", "profile": ""}));
        let (status, _, ben) = send_with(
            &app.router,
            Some(&app.ben),
            Method::GET,
            "/api/me",
            &[],
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_slice::<Value>(&ben).unwrap(),
            json!({"nickname": "Ben", "profile": ""})
        );
    }

    #[tokio::test]
    async fn updates_the_nickname_and_the_profile() {
        let app = app("update-me").await;
        let (status, ana) = app
            .send_json(
                Method::PATCH,
                "/api/me",
                Some(json!({"nickname": " Anaïs ", "profile": " Joue une barbare. "})),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            ana,
            json!({"nickname": "Anaïs", "profile": "Joue une barbare."})
        );
        let (status, ana) = app
            .send_json(Method::PATCH, "/api/me", Some(json!({"profile": ""})))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ana, json!({"nickname": "Anaïs", "profile": ""}));

        let (status, body) = app
            .send(Method::PATCH, "/api/me", Some(json!({"nickname": "BEN"})))
            .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body, b"nickname taken");
        for change in [
            json!({"nickname": "  "}),
            json!({"nickname": "x".repeat(crate::NICKNAME_MAX + 1)}),
            json!({"profile": "x".repeat(crate::PROFILE_MAX + 1)}),
            json!({"id": 7}),
        ] {
            let (status, _) = app
                .send(Method::PATCH, "/api/me", Some(change.clone()))
                .await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{change}");
        }
        let (_, ana) = app.send_json(Method::GET, "/api/me", None).await;
        assert_eq!(ana["nickname"], "Anaïs");
    }

    #[tokio::test]
    async fn lists_the_identities_by_provider() {
        let app = app("identities").await;
        let (status, identities) = app.send_json(Method::GET, "/api/me/identities", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            identities,
            json!([{"provider": "dex", "name": "Dex", "label": "Ana@mail.test"}])
        );
    }

    #[tokio::test]
    async fn creates_and_lists_characters_of_each_engine() {
        let app = app("characters").await;
        let uri = "/api/me/characters";

        let (status, brune) = app
            .send_json(
                Method::POST,
                uri,
                Some(json!({"name": "Brune", "engine": "fantasy"})),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(brune["id"], 1);
        assert_eq!(brune["name"], "Brune");
        assert_eq!(brune["engine"], "fantasy");
        assert_eq!(brune["version"], 1);
        assert_eq!(brune["document"]["engine"], "fantasy");
        assert_eq!(brune["document"]["version"], 1);

        let (status, max) = app
            .send_json(
                Method::POST,
                uri,
                Some(json!({"name": "Max", "engine": "contemporain"})),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(max["id"], 2);
        assert_eq!(max["engine"], "contemporain");
        assert_eq!(max["document"]["engine"], "contemporain");

        let (status, characters) = app.send_json(Method::GET, uri, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(characters, json!([brune, max]));
    }

    /// Ana's fantasy character 1.
    pub(super) async fn with_character(test: &str) -> App {
        let app = app(test).await;
        app.send(
            Method::POST,
            "/api/me/characters",
            Some(json!({"name": "Brune", "engine": "fantasy"})),
        )
        .await;
        app
    }

    pub(super) fn lhagva() -> Value {
        serde_json::from_str(include_str!("../../../document/tests/fixtures/lhagva.json")).unwrap()
    }

    #[tokio::test]
    async fn every_route_but_signing_in_needs_a_session() {
        let app = with_character("session").await;
        let routes = [
            (Method::GET, "/api/me"),
            (Method::PATCH, "/api/me"),
            (Method::GET, "/api/me/identities"),
            (Method::GET, "/api/me/characters"),
            (Method::POST, "/api/me/characters"),
            (Method::GET, "/api/characters/1"),
            (Method::PATCH, "/api/characters/1"),
            (Method::DELETE, "/api/characters/1"),
            (Method::PUT, "/api/characters/1/document"),
            (Method::GET, "/api/me/invitations"),
            (Method::POST, "/api/me/invitations"),
            (Method::DELETE, "/api/me/invitations/1"),
            (Method::POST, "/api/invitations/lookup"),
            (Method::GET, "/api/me/friends"),
            (Method::POST, "/api/me/friends"),
            (Method::GET, "/api/me/friends/1"),
            (Method::DELETE, "/api/me/friends/1"),
            (Method::GET, "/api/me/friends/1/characters"),
            (Method::GET, "/api/me/friends/1/characters/1"),
        ];
        for (method, uri) in routes {
            for cookie in [None, Some("palimpseste_session=unknown")] {
                let (status, _, _) =
                    send_with(&app.router, cookie, method.clone(), uri, &[], None).await;
                assert_eq!(
                    status,
                    StatusCode::UNAUTHORIZED,
                    "{method} {uri} {cookie:?}"
                );
            }
        }
        let (status, _, providers) = app.anonymous(Method::GET, "/api/auth/providers").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_slice::<Value>(&providers).unwrap(),
            json!([{"id": "dex", "name": "Dex"}])
        );
    }

    #[tokio::test]
    async fn characters_of_others_are_not_found() {
        let app = with_character("others").await;
        let ben = Some(app.ben.as_str());
        for (method, uri, body) in [
            (Method::GET, "/api/characters/1", None),
            (
                Method::PATCH,
                "/api/characters/1",
                Some(json!({"name": "X"})),
            ),
            (Method::DELETE, "/api/characters/1", None),
        ] {
            let (status, _, _) = send_with(&app.router, ben, method, uri, &[], body).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        }
        let (status, _, _) = send_with(
            &app.router,
            ben,
            Method::PUT,
            "/api/characters/1/document",
            &[(header::IF_MATCH, "\"1\"")],
            Some(lhagva()),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (_, _, list) = send_with(
            &app.router,
            ben,
            Method::GET,
            "/api/me/characters",
            &[],
            None,
        )
        .await;
        assert_eq!(serde_json::from_slice::<Value>(&list).unwrap(), json!([]));
        let (status, character) = app.send_json(Method::GET, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(character["name"], "Brune");
    }

    #[tokio::test]
    async fn signing_out_ends_the_session() {
        let app = app("logout").await;
        let (status, headers, _) = app.send_full(Method::POST, "/api/auth/logout", None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let cleared = headers[header::SET_COOKIE].to_str().unwrap();
        assert!(cleared.starts_with("palimpseste_session=;"), "{cleared}");
        assert!(cleared.contains("Max-Age=0"), "{cleared}");
        let (status, _) = app.send(Method::GET, "/api/me", None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let (status, _, _) = app.anonymous(Method::POST, "/api/auth/logout").await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn unknown_provider_is_not_found() {
        let app = app("unknown-provider").await;
        let (status, _, _) = app.anonymous(Method::GET, "/api/auth/nothing/login").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn replaces_a_document_at_its_version() {
        let app = with_character("replace").await;
        let (status, headers, _) = app.send_full(Method::GET, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::ETAG], "\"1\"");

        let (status, headers, _) = app.put_document(Some("\"1\""), lhagva()).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(headers[header::ETAG], "\"2\"");

        let (status, character) = app.send_json(Method::GET, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(character["version"], 2);
        assert_eq!(character["document"]["profil"], "cof.profil.barbare");
        assert_eq!(character["document"]["purse"], 70);

        let (status, headers, _) = app.put_document(Some("\"1\""), lhagva()).await;
        assert_eq!(status, StatusCode::PRECONDITION_FAILED);
        assert_eq!(headers[header::ETAG], "\"2\"");
        let (status, _, _) = app.put_document(None, lhagva()).await;
        assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
        let (status, _, _) = app.put_document(Some("*"), lhagva()).await;
        assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    }

    #[tokio::test]
    async fn stores_a_rule_breaking_document() {
        let app = with_character("rule-breaking").await;
        let mut document = lhagva();
        document["seriesValues"]["FOR"] = json!(12);
        document["voies"] = json!([{"id": "cof.combat", "ranks": [3, 9]}]);
        document["purse"] = json!(-500);
        let (status, _, _) = app.put_document(Some("\"1\""), document).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn rejects_documents_it_cannot_read() {
        let app = with_character("unreadable").await;
        let with = |pointer: &str, value: Value| {
            let mut document = lhagva();
            match document.pointer_mut(pointer) {
                Some(field) => *field = value,
                None => document[&pointer[1..]] = value,
            }
            document
        };
        for (case, document) in [
            (
                "unknown profil",
                with("/profil", json!("cof.profil.paladin")),
            ),
            (
                "unknown weapon",
                with("/items/0/id", json!("cof.weapon.laser")),
            ),
            ("unsupported version", with("/version", json!(2))),
            (
                "other engine",
                json!({"engine": "contemporain", "version": 1}),
            ),
            ("unknown engine", with("/engine", json!("pathfinder"))),
            ("negative rank", with("/voies/0/ranks", json!([-1]))),
            ("empty item name", with("/items/3/name", json!(""))),
            (
                "blank waiver reason",
                with(
                    "/waivers",
                    json!([{
                        "rule": "COF.CRE.EQUIPMENT",
                        "subject": {"kind": "character"},
                        "author": "gm",
                        "reason": " "
                    }]),
                ),
            ),
        ] {
            let (status, _, _) = app.put_document(Some("\"1\""), document).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{case}");
        }
        let (status, character) = app.send_json(Method::GET, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(character["version"], 1);
    }

    #[tokio::test]
    async fn rejects_malformed_json() {
        let app = with_character("malformed").await;
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/api/characters/1/document")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::IF_MATCH, "\"1\"")
            .header(header::COOKIE, &app.ana)
            .body(Body::from("{\"engine\":"))
            .unwrap();
        let response = app.router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn renames_and_deletes_characters() {
        let app = with_character("rename").await;
        let (status, character) = app
            .send_json(
                Method::PATCH,
                "/api/characters/1",
                Some(json!({"name": "Bruna"})),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(character["name"], "Bruna");
        let (status, _) = app.send(Method::DELETE, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        for (method, body) in [
            (Method::GET, None),
            (Method::DELETE, None),
            (Method::PATCH, Some(json!({"name": "X"}))),
        ] {
            let (status, _) = app.send(method, "/api/characters/1", body).await;
            assert_eq!(status, StatusCode::NOT_FOUND);
        }
        let (status, _, _) = app.put_document(Some("\"1\""), lhagva()).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn rejects_blank_names() {
        let app = with_character("blank-names").await;
        for (method, uri, body) in [
            (
                Method::POST,
                "/api/me/characters",
                json!({"name": "", "engine": "fantasy"}),
            ),
            (Method::PATCH, "/api/characters/1", json!({"name": "  "})),
        ] {
            let (status, _) = app.send(method, uri, Some(body)).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{uri}");
        }
    }

    #[tokio::test]
    async fn unknown_engine_is_rejected() {
        let app = app("unknown-engine").await;
        let character = json!({"name": "Brune", "engine": "pathfinder"});
        let (status, _) = app
            .send(Method::POST, "/api/me/characters", Some(character))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn serves_the_frontend() {
        let app = app("frontend").await;
        let (status, _, body) = app.anonymous(Method::GET, "/app.js").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"// app");
        let (status, _, body) = app.anonymous(Method::GET, "/").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"<p>index</p>");
        let (status, _, body) = app.anonymous(Method::GET, "/characters/1").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"<p>index</p>");
    }

    #[tokio::test]
    async fn unknown_api_paths_are_not_found() {
        let app = app("unknown-api").await;
        for uri in [
            "/api",
            "/api/",
            "/api/nothing",
            "/api/me/nothing",
            "/api/auth/nothing",
            "/api/users",
        ] {
            let (status, _, body) = app.send_full(Method::GET, uri, None).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
            assert!(body.is_empty(), "{uri}");
        }
    }

    #[tokio::test]
    async fn healthy_without_a_session() {
        let app = app("healthz").await;
        let (status, headers, body) = app.anonymous(Method::GET, "/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        assert_eq!(body, b"ok");
        let (status, _, body) = app.anonymous(Method::HEAD, "/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"");
    }

    #[tokio::test]
    async fn unhealthy_is_unavailable_without_detail() {
        let mut service = MockPalimpsesteService::new();
        service
            .expect_health()
            .returning(|| Err(anyhow!("disk I/O error").into()));
        let router = HttpApi::new(
            Arc::new(service),
            &frontend_dir("unhealthy"),
            "http://localhost:3000",
        )
        .router();
        let (status, headers, body) =
            send_with(&router, None, Method::GET, "/healthz", &[], None).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        assert_eq!(body, b"");
    }

    #[tokio::test]
    async fn technical_failures_are_internal_errors_without_detail() {
        let mut service = MockPalimpsesteService::new();
        service.expect_session_user().returning(|_| {
            Ok(User {
                id: UserId(1),
                nickname: Nickname::parse("Ana").unwrap(),
                profile: Profile::default(),
            })
        });
        service
            .expect_characters()
            .returning(|_| Err(anyhow!("the database is gone").into()));
        let router = HttpApi::new(
            Arc::new(service),
            &frontend_dir("internal"),
            "http://localhost:3000",
        )
        .router();
        let (status, _, body) = send_with(
            &router,
            Some("palimpseste_session=token"),
            Method::GET,
            "/api/me/characters",
            &[],
            None,
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, b"");
    }
}
