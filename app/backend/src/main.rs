//! Palimpseste backend, a proof of concept: stores users and their list of
//! characters in memory, each character tied to its engine, and serves the
//! front end.
//!
//! Configuration through the environment:
//! - `PALIMPSESTE_ADDR`: address to listen on (default `127.0.0.1:3000`);
//! - `PALIMPSESTE_FRONTEND_DIR`: directory of the built front end (default
//!   `app/frontend/dist`).

mod api;
mod store;

use std::{env, path::PathBuf, sync::Arc};

use store::Store;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = env::var("PALIMPSESTE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let frontend_dir: PathBuf = env::var_os("PALIMPSESTE_FRONTEND_DIR")
        .map_or_else(|| "app/frontend/dist".into(), PathBuf::from);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        api::router(Arc::new(Store::default()), &frontend_dir),
    )
    .await
}
