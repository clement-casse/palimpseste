//! The built front end (ADR BE-FRONTEND): its files, falling back to its
//! `index.html` so the front end can route itself.

use std::path::Path;

use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, header},
    middleware::{self, Next},
    response::Response,
};
use tower_http::services::{ServeDir, ServeFile};

/// Serves `frontend_dir`. Missing files under `/assets` are not found.
pub(super) fn router(frontend_dir: &Path) -> Router {
    Router::new()
        .nest_service("/assets", ServeDir::new(frontend_dir.join("assets")))
        .fallback_service(
            ServeDir::new(frontend_dir).fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        .layer(middleware::from_fn(cache_control))
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::*;
    use crate::api::http::tests::frontend_dir;

    async fn get(router: &Router, uri: &str) -> Response {
        router
            .clone()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn serves_the_pwa_files_with_their_type() {
        let router = router(&frontend_dir("pwa-types"));
        for (uri, content_type) in [
            ("/assets/engine-abc123.wasm", "application/wasm"),
            ("/manifest.webmanifest", "application/manifest+json"),
            ("/sw.js", "text/javascript"),
        ] {
            let response = get(&router, uri).await;
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                content_type,
                "{uri}"
            );
        }
    }

    #[tokio::test]
    async fn keeps_hashed_assets_and_revalidates_the_rest() {
        let router = router(&frontend_dir("cache-control"));
        let response = get(&router, "/assets/engine-abc123.wasm").await;
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        for uri in [
            "/",
            "/sw.js",
            "/manifest.webmanifest",
            "/characters/1",
            "/assets/missing.js",
        ] {
            let response = get(&router, uri).await;
            assert_eq!(
                response.headers()[header::CACHE_CONTROL],
                "no-cache",
                "{uri}"
            );
        }
    }
}
