//! HTTP routes: the JSON API under `/api`, the front end everywhere else.
//!
//! The API serves, checks and stores (ADR 0007): it rejects a request it
//! cannot read, never a character breaking a rule. Store calls block, so
//! they run on `spawn_blocking`. Each request is logged with its status and
//! latency, each change to the store and each rejected request with why.

use std::{path::Path, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path as UrlPath, Request, State},
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

use crate::store::{self, Character, CharacterId, Replaced, Store, User, UserId};

#[derive(Debug, Deserialize)]
struct NewUser {
    name: String,
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
enum ApiError {
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
pub fn router(store: Arc<Store>, frontend_dir: &Path) -> Router {
    let not_found = || async { StatusCode::NOT_FOUND };
    let api = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{user_id}", get(get_user))
        .route(
            "/users/{user_id}/characters",
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
        .with_state(store);
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
async fn blocking<T: Send + 'static>(
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

async fn list_users(State(store): State<Arc<Store>>) -> ApiResult<Json<Vec<User>>> {
    blocking(store, Store::users).await.map(Json)
}

async fn create_user(
    State(store): State<Arc<Store>>,
    Json(new): Json<NewUser>,
) -> ApiResult<(StatusCode, Json<User>)> {
    let name = filled_name(&new.name)?;
    let user = blocking(store, move |store| store.create_user(&name)).await?;
    tracing::info!(user_id = %user.id, "user created");
    Ok((StatusCode::CREATED, Json(user)))
}

async fn get_user(
    State(store): State<Arc<Store>>,
    UrlPath(user_id): UrlPath<UserId>,
) -> ApiResult<Json<User>> {
    blocking(store, move |store| store.user(user_id))
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn list_characters(
    State(store): State<Arc<Store>>,
    UrlPath(user_id): UrlPath<UserId>,
) -> ApiResult<Json<Vec<Character>>> {
    blocking(store, move |store| store.characters(user_id))
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn create_character(
    State(store): State<Arc<Store>>,
    UrlPath(user_id): UrlPath<UserId>,
    Json(new): Json<NewCharacter>,
) -> ApiResult<(StatusCode, Json<Character>)> {
    let name = filled_name(&new.name)?;
    let character = blocking(store, move |store| {
        store.create_character(user_id, &name, new.engine)
    })
    .await?
    .ok_or(ApiError::NotFound)?;
    tracing::info!(character_id = %character.id, %user_id, engine = ?character.engine, "character created");
    Ok((StatusCode::CREATED, Json(character)))
}

async fn get_character(
    State(store): State<Arc<Store>>,
    UrlPath(character_id): UrlPath<CharacterId>,
) -> ApiResult<impl IntoResponse> {
    let character = blocking(store, move |store| store.character(character_id))
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(([(header::ETAG, etag(character.version))], Json(character)))
}

async fn rename_character(
    State(store): State<Arc<Store>>,
    UrlPath(character_id): UrlPath<CharacterId>,
    Json(rename): Json<Rename>,
) -> ApiResult<Json<Character>> {
    let name = filled_name(&rename.name)?;
    let character = blocking(store, move |store| {
        store.rename_character(character_id, &name)
    })
    .await?
    .ok_or(ApiError::NotFound)?;
    tracing::info!(%character_id, "character renamed");
    Ok(Json(character))
}

async fn delete_character(
    State(store): State<Arc<Store>>,
    UrlPath(character_id): UrlPath<CharacterId>,
) -> ApiResult<StatusCode> {
    if blocking(store, move |store| store.delete_character(character_id)).await? {
        tracing::info!(%character_id, "character deleted");
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Replaces a character's whole document, at the version `If-Match` names.
/// The document must be readable (ADR 0007, D6); its rules are not checked.
async fn put_document(
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
        store.replace_document(character_id, expected.unwrap_or(0), &document)
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

    fn app(test: &str) -> Router {
        router(Arc::new(Store::in_memory().unwrap()), &frontend_dir(test))
    }

    async fn send(
        app: &Router,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Vec<u8>) {
        let (status, _, body) = send_full(app, method, uri, body).await;
        (status, body)
    }

    async fn send_full(
        app: &Router,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, HeaderMap, Vec<u8>) {
        send_with(app, method, uri, &[], body).await
    }

    async fn send_with(
        app: &Router,
        method: Method,
        uri: &str,
        headers: &[(header::HeaderName, &str)],
        body: Option<Value>,
    ) -> (StatusCode, HeaderMap, Vec<u8>) {
        let request = headers.iter().fold(
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
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, headers, bytes.to_vec())
    }

    async fn send_json(
        app: &Router,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let (status, bytes) = send(app, method, uri, body).await;
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn creates_and_lists_users() {
        let app = app("users");
        let (status, ana) = send_json(
            &app,
            Method::POST,
            "/api/users",
            Some(json!({"name": "Ana"})),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(ana, json!({"id": 1, "name": "Ana"}));

        let (status, users) = send_json(&app, Method::GET, "/api/users", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(users, json!([ana]));

        let (status, user) = send_json(&app, Method::GET, "/api/users/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(user, ana);
    }

    #[tokio::test]
    async fn creates_and_lists_characters_of_each_engine() {
        let app = app("characters");
        send(
            &app,
            Method::POST,
            "/api/users",
            Some(json!({"name": "Ana"})),
        )
        .await;
        let uri = "/api/users/1/characters";

        let (status, brune) = send_json(
            &app,
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

        let (status, max) = send_json(
            &app,
            Method::POST,
            uri,
            Some(json!({"name": "Max", "engine": "contemporain"})),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(max["id"], 2);
        assert_eq!(max["engine"], "contemporain");
        assert_eq!(max["document"]["engine"], "contemporain");

        let (status, characters) = send_json(&app, Method::GET, uri, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(characters, json!([brune, max]));
    }

    /// Creates user 1 and their fantasy character 1.
    async fn with_character(test: &str) -> Router {
        let app = app(test);
        send(
            &app,
            Method::POST,
            "/api/users",
            Some(json!({"name": "Ana"})),
        )
        .await;
        send(
            &app,
            Method::POST,
            "/api/users/1/characters",
            Some(json!({"name": "Brune", "engine": "fantasy"})),
        )
        .await;
        app
    }

    fn lhagva() -> Value {
        serde_json::from_str(include_str!("../../document/tests/fixtures/lhagva.json")).unwrap()
    }

    async fn put_document(
        app: &Router,
        version: Option<&str>,
        document: Value,
    ) -> (StatusCode, HeaderMap, Vec<u8>) {
        let headers: Vec<(header::HeaderName, &str)> =
            version.map(|v| (header::IF_MATCH, v)).into_iter().collect();
        send_with(
            app,
            Method::PUT,
            "/api/characters/1/document",
            &headers,
            Some(document),
        )
        .await
    }

    #[tokio::test]
    async fn replaces_a_document_at_its_version() {
        let app = with_character("replace").await;
        let (status, headers, _) = send_full(&app, Method::GET, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::ETAG], "\"1\"");

        let (status, headers, _) = put_document(&app, Some("\"1\""), lhagva()).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(headers[header::ETAG], "\"2\"");

        let (status, character) = send_json(&app, Method::GET, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(character["version"], 2);
        assert_eq!(character["document"]["profil"], "cof.profil.barbare");
        assert_eq!(character["document"]["purse"], 70);

        let (status, headers, _) = put_document(&app, Some("\"1\""), lhagva()).await;
        assert_eq!(status, StatusCode::PRECONDITION_FAILED);
        assert_eq!(headers[header::ETAG], "\"2\"");
        let (status, _, _) = put_document(&app, None, lhagva()).await;
        assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
        let (status, _, _) = put_document(&app, Some("*"), lhagva()).await;
        assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    }

    #[tokio::test]
    async fn stores_a_rule_breaking_document() {
        let app = with_character("rule-breaking").await;
        let mut document = lhagva();
        document["seriesValues"]["FOR"] = json!(12);
        document["voies"] = json!([{"id": "cof.combat", "ranks": [3, 9]}]);
        document["purse"] = json!(-500);
        let (status, _, _) = put_document(&app, Some("\"1\""), document).await;
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
            let (status, _, _) = put_document(&app, Some("\"1\""), document).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{case}");
        }
        let (status, character) = send_json(&app, Method::GET, "/api/characters/1", None).await;
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
            .body(Body::from("{\"engine\":"))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn renames_and_deletes_characters() {
        let app = with_character("rename").await;
        let (status, character) = send_json(
            &app,
            Method::PATCH,
            "/api/characters/1",
            Some(json!({"name": "Bruna"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(character["name"], "Bruna");
        let (status, _) = send(&app, Method::DELETE, "/api/characters/1", None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        for (method, body) in [
            (Method::GET, None),
            (Method::DELETE, None),
            (Method::PATCH, Some(json!({"name": "X"}))),
        ] {
            let (status, _) = send(&app, method, "/api/characters/1", body).await;
            assert_eq!(status, StatusCode::NOT_FOUND);
        }
        let (status, _, _) = put_document(&app, Some("\"1\""), lhagva()).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn rejects_blank_names() {
        let app = with_character("blank-names").await;
        for (method, uri, body) in [
            (Method::POST, "/api/users", json!({"name": " "})),
            (
                Method::POST,
                "/api/users/1/characters",
                json!({"name": "", "engine": "fantasy"}),
            ),
            (Method::PATCH, "/api/characters/1", json!({"name": "  "})),
        ] {
            let (status, _) = send(&app, method, uri, Some(body)).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{uri}");
        }
    }

    #[tokio::test]
    async fn unknown_user_is_not_found() {
        let app = app("unknown-user");
        let character = json!({"name": "Brune", "engine": "fantasy"});
        let (status, _) = send(&app, Method::GET, "/api/users/7", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = send(&app, Method::GET, "/api/users/7/characters", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = send(
            &app,
            Method::POST,
            "/api/users/7/characters",
            Some(character),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_engine_is_rejected() {
        let app = app("unknown-engine");
        send(
            &app,
            Method::POST,
            "/api/users",
            Some(json!({"name": "Ana"})),
        )
        .await;
        let character = json!({"name": "Brune", "engine": "pathfinder"});
        let (status, _) = send(
            &app,
            Method::POST,
            "/api/users/1/characters",
            Some(character),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn serves_the_frontend() {
        let app = app("frontend");
        let (status, body) = send(&app, Method::GET, "/app.js", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"// app");
        let (status, body) = send(&app, Method::GET, "/", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"<p>index</p>");
        let (status, body) = send(&app, Method::GET, "/characters/1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"<p>index</p>");
    }

    #[tokio::test]
    async fn unknown_api_paths_are_not_found() {
        let app = app("unknown-api");
        for uri in ["/api", "/api/", "/api/nothing", "/api/users/1/nothing"] {
            let (status, body) = send(&app, Method::GET, uri, None).await;
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
            let (status, headers, _) = send_full(&app, Method::GET, uri, None).await;
            assert_eq!(status, StatusCode::OK, "{uri}");
            assert_eq!(headers[header::CONTENT_TYPE], content_type, "{uri}");
        }
    }

    #[tokio::test]
    async fn keeps_hashed_assets_and_revalidates_the_rest() {
        let app = app("cache-control");
        let (_, headers, _) =
            send_full(&app, Method::GET, "/assets/engine-abc123.wasm", None).await;
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
            let (_, headers, _) = send_full(&app, Method::GET, uri, None).await;
            assert_eq!(headers[header::CACHE_CONTROL], "no-cache", "{uri}");
        }
    }
}
