//! HTTP routes: the JSON API under `/api`, the front end everywhere else.
//!
//! The API serves, checks and stores (ADR 0007): it rejects a request it
//! cannot read, never a character breaking a rule. Signing in is under
//! `/api/auth` (ADR 0009); every other route needs a session and reaches
//! the signed-in user's characters only. Store calls block, so they run on
//! `spawn_blocking`. Each request is logged with its status and
//! latency, each change to the store and each rejected request with why.

use std::{path::Path, sync::Arc};

use axum::{
    Json, Router,
    extract::{FromRef, Path as UrlPath, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, put},
};
use palimpseste_document::{Document, Engine};
use serde::Deserialize;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{
    auth::{self, Auth, CurrentUser},
    store::{self, Character, CharacterId, Replaced, Store, User},
};

/// What the handlers share: the store and the sign-in configuration.
#[derive(Debug, Clone)]
pub struct AppState {
    pub store: Arc<Store>,
    pub auth: Arc<Auth>,
}

impl FromRef<AppState> for Arc<Store> {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

impl FromRef<AppState> for Arc<Auth> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
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

/// Why a request failed, besides the JSON the extractors reject (`400` for
/// malformed JSON, `422` for an unknown shape).
#[derive(Debug)]
pub enum ApiError {
    /// No live session.
    Unauthorized,
    /// Unknown, or not the signed-in user's.
    NotFound,
    /// Readable JSON whose content cannot be stored (ADR 0007, D6).
    Unprocessable(String),
    /// A document replaced without `If-Match`.
    PreconditionRequired,
    /// A document replaced at a stale version: the current one is given.
    PreconditionFailed(i64),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Self::NotFound => StatusCode::NOT_FOUND.into_response(),
            Self::Unprocessable(reason) => {
                tracing::warn!(%reason, "unprocessable request");
                (StatusCode::UNPROCESSABLE_ENTITY, reason).into_response()
            }
            Self::PreconditionRequired => {
                tracing::warn!("document replaced without If-Match");
                (StatusCode::PRECONDITION_REQUIRED, "If-Match is required").into_response()
            }
            Self::PreconditionFailed(version) => {
                tracing::info!(current = version, "document replaced at a stale version");
                (
                    StatusCode::PRECONDITION_FAILED,
                    [(header::ETAG, etag(version))],
                )
                    .into_response()
            }
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

type ApiResult<T> = Result<T, ApiError>;

/// Builds the router. Paths outside `/api` are served from `frontend_dir`,
/// falling back to its `index.html` so the front end can route itself.
/// Unknown paths under `/api` and missing files under `/assets` are not
/// found. Every request is traced.
pub fn router(store: Arc<Store>, auth: Arc<Auth>, frontend_dir: &Path) -> Router {
    let not_found = || async { StatusCode::NOT_FOUND };
    let api = Router::new()
        .nest("/auth", auth::router())
        .route("/me", get(me))
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
        .fallback(not_found)
        .with_state(AppState { store, auth });
    let frontend = Router::new()
        .nest_service("/assets", ServeDir::new(frontend_dir.join("assets")))
        .fallback_service(
            ServeDir::new(frontend_dir).fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        .layer(middleware::from_fn(cache_control));
    Router::new()
        .nest("/api", api)
        // `nest` leaves the bare trailing slash to the front end.
        .route("/api/", any(not_found))
        .fallback_service(frontend)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}

/// Sets how long browsers may keep a front end file. The build names the
/// files under `/assets/` after their content, so they never change and are
/// kept for good. Every other file (`index.html`, the service worker, the
/// manifest) is revalidated on each request, so a new build reaches users.
async fn cache_control(request: Request, next: Next) -> Response {
    let hashed = request.uri().path().starts_with("/assets/");
    let mut response = next.run(request).await;
    let policy = if hashed && response.status().is_success() {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(policy));
    response
}

/// Runs `query` on the store, off the async runtime.
pub async fn blocking<T: Send + 'static>(
    store: Arc<Store>,
    query: impl FnOnce(&Store) -> store::Result<T> + Send + 'static,
) -> ApiResult<T> {
    match tokio::task::spawn_blocking(move || query(&store)).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => {
            tracing::error!(%error, "store error");
            Err(ApiError::Internal)
        }
        Err(error) => {
            tracing::error!(%error, "store task failed");
            Err(ApiError::Internal)
        }
    }
}

/// A name, unless blank.
fn filled_name(name: &str) -> ApiResult<String> {
    let name = name.trim();
    if name.is_empty() {
        Err(ApiError::Unprocessable("empty name".into()))
    } else {
        Ok(name.to_owned())
    }
}

/// The `ETag` of a document version.
fn etag(version: i64) -> String {
    format!("\"{version}\"")
}

/// The version `If-Match` expects; `None` when it names no version.
fn if_match(headers: &HeaderMap) -> ApiResult<Option<i64>> {
    let value = headers
        .get(header::IF_MATCH)
        .ok_or(ApiError::PreconditionRequired)?;
    Ok(value
        .to_str()
        .ok()
        .and_then(|value| value.trim().strip_prefix('"')?.strip_suffix('"'))
        .and_then(|version| version.parse().ok()))
}

async fn me(CurrentUser(user): CurrentUser) -> Json<User> {
    Json(user)
}

async fn list_characters(
    CurrentUser(user): CurrentUser,
    State(store): State<Arc<Store>>,
) -> ApiResult<Json<Vec<Character>>> {
    blocking(store, move |store| store.characters(user.id))
        .await
        .map(Json)
}

async fn create_character(
    CurrentUser(user): CurrentUser,
    State(store): State<Arc<Store>>,
    Json(new): Json<NewCharacter>,
) -> ApiResult<(StatusCode, Json<Character>)> {
    let name = filled_name(&new.name)?;
    let character = blocking(store, move |store| {
        store.create_character(user.id, &name, new.engine)
    })
    .await?;
    tracing::info!(character_id = %character.id, user_id = %user.id, engine = ?character.engine, "character created");
    Ok((StatusCode::CREATED, Json(character)))
}

async fn get_character(
    CurrentUser(user): CurrentUser,
    State(store): State<Arc<Store>>,
    UrlPath(character_id): UrlPath<CharacterId>,
) -> ApiResult<impl IntoResponse> {
    let character = blocking(store, move |store| store.character(user.id, character_id))
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(([(header::ETAG, etag(character.version))], Json(character)))
}

async fn rename_character(
    CurrentUser(user): CurrentUser,
    State(store): State<Arc<Store>>,
    UrlPath(character_id): UrlPath<CharacterId>,
    Json(rename): Json<Rename>,
) -> ApiResult<Json<Character>> {
    let name = filled_name(&rename.name)?;
    let character = blocking(store, move |store| {
        store.rename_character(user.id, character_id, &name)
    })
    .await?
    .ok_or(ApiError::NotFound)?;
    tracing::info!(%character_id, "character renamed");
    Ok(Json(character))
}

async fn delete_character(
    CurrentUser(user): CurrentUser,
    State(store): State<Arc<Store>>,
    UrlPath(character_id): UrlPath<CharacterId>,
) -> ApiResult<StatusCode> {
    if blocking(store, move |store| {
        store.delete_character(user.id, character_id)
    })
    .await?
    {
        tracing::info!(%character_id, "character deleted");
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Replaces a character's whole document, at the version `If-Match` names.
/// The document must be readable (ADR 0007, D6); its rules are not checked.
async fn put_document(
    CurrentUser(user): CurrentUser,
    State(store): State<Arc<Store>>,
    UrlPath(character_id): UrlPath<CharacterId>,
    headers: HeaderMap,
    Json(document): Json<Document>,
) -> ApiResult<impl IntoResponse> {
    let expected = if_match(&headers)?;
    document
        .check()
        .map_err(|error| ApiError::Unprocessable(error.to_string()))?;
    let replaced = blocking(store, move |store| {
        // A version `If-Match` cannot name never matches: version 0 is
        // never stored.
        store.replace_document(user.id, character_id, expected.unwrap_or(0), &document)
    })
    .await?;
    match replaced {
        Replaced::Stored(version) => {
            tracing::info!(%character_id, version, "document stored");
            Ok((StatusCode::NO_CONTENT, [(header::ETAG, etag(version))]))
        }
        Replaced::NotFound => Err(ApiError::NotFound),
        Replaced::Stale(version) => Err(ApiError::PreconditionFailed(version)),
        Replaced::OtherEngine => Err(ApiError::Unprocessable(
            "the document is of another engine than the character".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, Method, Request},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::auth::{Provider, session_cookie};

    /// A frontend directory unique to the test, holding an `index.html`.
    fn frontend_dir(test: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("palimpseste-backend-{}-{test}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("index.html"), "<p>index</p>").unwrap();
        fs::write(dir.join("app.js"), "// app").unwrap();
        fs::write(dir.join("sw.js"), "// service worker").unwrap();
        fs::write(dir.join("manifest.webmanifest"), "{}").unwrap();
        fs::write(dir.join("assets/engine-abc123.wasm"), b"\0asm").unwrap();
        dir
    }

    /// The application, with users Ana (1) and Ben (2) signed in.
    struct App {
        router: Router,
        ana: String,
        ben: String,
    }

    fn app(test: &str) -> App {
        let store = Arc::new(Store::in_memory().unwrap());
        let mut cookies = ["Ana", "Ben"].map(|name| {
            let (user, _) = store
                .user_for_identity("https://issuer.test", name, name)
                .unwrap();
            session_cookie(&store, user.id)
        });
        let provider =
            Provider::new("dex", "Dex", "https://issuer.test", "client", "secret").unwrap();
        let auth = Auth::new("http://localhost:3000", vec![provider]).unwrap();
        App {
            router: router(store, Arc::new(auth), &frontend_dir(test)),
            ana: std::mem::take(&mut cookies[0]),
            ben: std::mem::take(&mut cookies[1]),
        }
    }

    /// Sends a request with `cookie`, if any, and `headers`.
    async fn send_with(
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
        let app = app("me");
        let (status, ana) = app.send_json(Method::GET, "/api/me", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ana, json!({"id": 1, "name": "Ana"}));
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
            json!({"id": 2, "name": "Ben"})
        );
    }

    #[tokio::test]
    async fn creates_and_lists_characters_of_each_engine() {
        let app = app("characters");
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
    async fn with_character(test: &str) -> App {
        let app = app(test);
        app.send(
            Method::POST,
            "/api/me/characters",
            Some(json!({"name": "Brune", "engine": "fantasy"})),
        )
        .await;
        app
    }

    fn lhagva() -> Value {
        serde_json::from_str(include_str!("../../document/tests/fixtures/lhagva.json")).unwrap()
    }

    #[tokio::test]
    async fn every_route_but_signing_in_needs_a_session() {
        let app = with_character("session").await;
        let routes = [
            (Method::GET, "/api/me"),
            (Method::GET, "/api/me/characters"),
            (Method::POST, "/api/me/characters"),
            (Method::GET, "/api/characters/1"),
            (Method::PATCH, "/api/characters/1"),
            (Method::DELETE, "/api/characters/1"),
            (Method::PUT, "/api/characters/1/document"),
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
        let app = app("logout");
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
        let app = app("unknown-provider");
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
        let app = app("unknown-engine");
        let character = json!({"name": "Brune", "engine": "pathfinder"});
        let (status, _) = app
            .send(Method::POST, "/api/me/characters", Some(character))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn serves_the_frontend() {
        let app = app("frontend");
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
        let app = app("unknown-api");
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
    async fn serves_the_pwa_files_with_their_type() {
        let app = app("pwa-types");
        for (uri, content_type) in [
            ("/assets/engine-abc123.wasm", "application/wasm"),
            ("/manifest.webmanifest", "application/manifest+json"),
            ("/sw.js", "text/javascript"),
        ] {
            let (status, headers, _) = app.anonymous(Method::GET, uri).await;
            assert_eq!(status, StatusCode::OK, "{uri}");
            assert_eq!(headers[header::CONTENT_TYPE], content_type, "{uri}");
        }
    }

    #[tokio::test]
    async fn keeps_hashed_assets_and_revalidates_the_rest() {
        let app = app("cache-control");
        let (_, headers, _) = app
            .anonymous(Method::GET, "/assets/engine-abc123.wasm")
            .await;
        assert_eq!(
            headers[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        for uri in [
            "/",
            "/sw.js",
            "/manifest.webmanifest",
            "/characters/1",
            "/assets/missing.js",
        ] {
            let (_, headers, _) = app.anonymous(Method::GET, uri).await;
            assert_eq!(headers[header::CACHE_CONTROL], "no-cache", "{uri}");
        }
    }
}
