//! Palimpseste backend: serves the front end, and checks and stores users
//! and their characters with their choices documents in SQLite. The rules
//! run in the browser (ADR 0007).
//!
//! Configuration through the environment:
//! - `PALIMPSESTE_ADDR`: address to listen on (default `127.0.0.1:3000`);
//! - `PALIMPSESTE_FRONTEND_DIR`: directory of the built front end (default
//!   `app/frontend/dist`);
//! - `PALIMPSESTE_DATABASE`: the SQLite database file, created if missing
//!   (default `palimpseste.sqlite3`);
//! - `RUST_LOG`: what to log, as a `tracing-subscriber` filter (default
//!   `info`).

mod api;
mod store;

use std::{env, path::PathBuf, sync::Arc};

use store::Store;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let addr = env::var("PALIMPSESTE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let frontend_dir: PathBuf = env::var_os("PALIMPSESTE_FRONTEND_DIR")
        .map_or_else(|| "app/frontend/dist".into(), PathBuf::from);
    let database: PathBuf = env::var_os("PALIMPSESTE_DATABASE")
        .map_or_else(|| "palimpseste.sqlite3".into(), PathBuf::from);
    let store = Store::open(&database).map_err(|error| {
        tracing::error!(database = %database.display(), %error, "cannot open the database");
        std::io::Error::other(error)
    })?;
    tracing::info!(database = %database.display(), "database opened");
    if !frontend_dir.join("index.html").is_file() {
        tracing::warn!(
            frontend_dir = %frontend_dir.display(),
            "no index.html in the front end directory: build it with `mise run frontend:build`"
        );
    }
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|error| {
            tracing::error!(%addr, %error, "cannot listen");
            error
        })?;
    tracing::info!(addr = %listener.local_addr()?, frontend_dir = %frontend_dir.display(), "listening");
    axum::serve(listener, api::router(Arc::new(store), &frontend_dir)).await
}
