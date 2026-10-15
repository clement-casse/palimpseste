//! HTTP routes: the JSON API under `/api`, the front end everywhere else.

use std::{path::Path, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path as UrlPath, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::{any, get},
};
use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};

use crate::store::{Character, Engine, Store, User, UserId};

#[derive(Debug, Deserialize)]
struct NewUser {
    name: String,
}

#[derive(Debug, Deserialize)]
struct NewCharacter {
    name: String,
    engine: Engine,
}

/// Builds the router. Paths outside `/api` are served from `frontend_dir`,
/// falling back to its `index.html` so the front end can route itself.
/// Unknown paths under `/api` and missing files under `/assets` are not
/// found.
pub fn router(store: Arc<Store>, frontend_dir: &Path) -> Router {
    let not_found = || async { StatusCode::NOT_FOUND };
    let api = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{user_id}", get(get_user))
        .route(
            "/users/{user_id}/characters",
            get(list_characters).post(create_character),
        )
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

async fn list_users(State(store): State<Arc<Store>>) -> Json<Vec<User>> {
    Json(store.users())
}

async fn create_user(
    State(store): State<Arc<Store>>,
    Json(new): Json<NewUser>,
) -> (StatusCode, Json<User>) {
    (StatusCode::CREATED, Json(store.create_user(new.name)))
}

async fn get_user(
    State(store): State<Arc<Store>>,
    UrlPath(user_id): UrlPath<UserId>,
) -> Result<Json<User>, StatusCode> {
    store.user(user_id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn list_characters(
    State(store): State<Arc<Store>>,
    UrlPath(user_id): UrlPath<UserId>,
) -> Result<Json<Vec<Character>>, StatusCode> {
    store
        .characters(user_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_character(
    State(store): State<Arc<Store>>,
    UrlPath(user_id): UrlPath<UserId>,
    Json(new): Json<NewCharacter>,
) -> Result<(StatusCode, Json<Character>), StatusCode> {
    store
        .create_character(user_id, new.name, new.engine)
        .map(|character| (StatusCode::CREATED, Json(character)))
        .ok_or(StatusCode::NOT_FOUND)
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
        router(Arc::new(Store::default()), &frontend_dir(test))
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
        let request = Request::builder().method(method).uri(uri);
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
        assert_eq!(
            brune,
            json!({"id": 1, "name": "Brune", "engine": "fantasy"})
        );

        let (status, max) = send_json(
            &app,
            Method::POST,
            uri,
            Some(json!({"name": "Max", "engine": "contemporain"})),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(
            max,
            json!({"id": 2, "name": "Max", "engine": "contemporain"})
        );

        let (status, characters) = send_json(&app, Method::GET, uri, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(characters, json!([brune, max]));
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
