//! Palimpseste backend: serves the front end, and checks and stores users
//! and their characters with their choices documents in SQLite. The rules
//! run in the browser (ADR 0007).
//!
//! Configuration through the environment:
//! - `PALIMPSESTE_ADDR`: address to listen on (default `127.0.0.1:3000`);
//! - `PALIMPSESTE_FRONTEND_DIR`: directory of the built front end (default
//!   `app/frontend/dist`);
//! - `PALIMPSESTE_DATABASE`: the SQLite database file, created if missing
//!   (default `palimpseste.sqlite3`).

mod api;
mod store;

use std::{env, path::PathBuf, sync::Arc};

use store::Store;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = env::var("PALIMPSESTE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let frontend_dir: PathBuf = env::var_os("PALIMPSESTE_FRONTEND_DIR")
        .map_or_else(|| "app/frontend/dist".into(), PathBuf::from);
    let database: PathBuf = env::var_os("PALIMPSESTE_DATABASE")
        .map_or_else(|| "palimpseste.sqlite3".into(), PathBuf::from);
    let store = Store::open(&database).map_err(std::io::Error::other)?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, api::router(Arc::new(store), &frontend_dir)).await
}
