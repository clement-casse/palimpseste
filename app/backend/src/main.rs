//! The Palimpseste server: the composition root of `palimpseste_backend`
//! (ADR 0012). It reads the configuration, builds the repository, the
//! identity providers, the service and its HTTP adapter, and listens.
//! `--help` lists the options and their environment variables.
//!
//! Each provider named by `--oidc-providers` is set by
//! `PALIMPSESTE_OIDC_<ID>_NAME`, `_ISSUER`, `_CLIENT_ID`, `_CLIENT_SECRET`
//! and optionally `_SCOPES` (besides `openid`, default `profile email`),
//! `<ID>` upper-cased with dashes as underscores (ADR 0009). `RUST_LOG`
//! filters the logs, as a `tracing-subscriber` filter (default `info`).

use std::{env, path::PathBuf, sync::Arc};

use anyhow::{Context as _, anyhow};
use clap::Parser;
use palimpseste_backend::{
    HttpApi, IdentityProviders as _, OidcProviders, PalimpsesteServiceImpl, ProviderConfig,
    SqliteRepo,
};
use tracing_subscriber::EnvFilter;

/// Serves the Palimpseste application: the front end, sign-in through OIDC
/// providers, and the users' characters.
#[derive(Debug, Parser)]
#[command(version)]
struct Config {
    /// Address to listen on.
    #[arg(long, env = "PALIMPSESTE_ADDR", default_value = "127.0.0.1:3000")]
    addr: String,
    /// Directory of the built front end.
    #[arg(
        long,
        env = "PALIMPSESTE_FRONTEND_DIR",
        default_value = "app/frontend/dist"
    )]
    frontend_dir: PathBuf,
    /// The SQLite database file, created if missing.
    #[arg(
        long,
        env = "PALIMPSESTE_DATABASE",
        default_value = "palimpseste.sqlite3"
    )]
    database: PathBuf,
    /// The URL users reach the server at, base of the redirect URIs
    /// registered at the providers (`{url}/api/auth/{id}/callback`);
    /// `https` makes cookies `Secure`.
    #[arg(
        long,
        env = "PALIMPSESTE_PUBLIC_URL",
        default_value = "http://127.0.0.1:3000"
    )]
    public_url: String,
    /// Comma-separated ids of the OIDC providers (lower-case letters,
    /// digits and dashes), each set by its `PALIMPSESTE_OIDC_<ID>_*`
    /// variables.
    #[arg(long, env = "PALIMPSESTE_OIDC_PROVIDERS", value_delimiter = ',')]
    oidc_providers: Vec<String>,
}

impl Config {
    /// The configuration of each provider, from the environment.
    fn providers(&self) -> anyhow::Result<Vec<ProviderConfig>> {
        self.oidc_providers
            .iter()
            .map(|id| id.trim())
            .filter(|id| !id.is_empty())
            .map(provider_from_env)
            .collect()
    }
}

/// Provider `id`, from `PALIMPSESTE_OIDC_<ID>_NAME`, `_ISSUER`,
/// `_CLIENT_ID`, `_CLIENT_SECRET`, and `_SCOPES` if set. Their names depend
/// on the id, so clap cannot declare them.
fn provider_from_env(id: &str) -> anyhow::Result<ProviderConfig> {
    let prefix = format!(
        "PALIMPSESTE_OIDC_{}_",
        id.to_ascii_uppercase().replace('-', "_")
    );
    let var = |name: &str| {
        env::var(format!("{prefix}{name}")).map_err(|_| anyhow!("{prefix}{name} is not set"))
    };
    let provider = ProviderConfig::new(
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let providers = OidcProviders::new(&config.public_url, config.providers()?)
        .context("invalid sign-in configuration")?;
    let ids: Vec<String> = providers
        .list()
        .into_iter()
        .map(|provider| provider.id.to_string())
        .collect();
    if ids.is_empty() {
        tracing::warn!("no OIDC provider: nobody can sign in, set PALIMPSESTE_OIDC_PROVIDERS");
    } else {
        tracing::info!(providers = ?ids, "sign-in providers");
    }
    let repo = SqliteRepo::open(&config.database)
        .await
        .with_context(|| format!("cannot open the database {}", config.database.display()))?;
    tracing::info!(database = %config.database.display(), "database opened");
    if !config.frontend_dir.join("index.html").is_file() {
        tracing::warn!(
            frontend_dir = %config.frontend_dir.display(),
            "no index.html in the front end directory: build it with `mise run frontend:build`"
        );
    }

    let service = Arc::new(PalimpsesteServiceImpl::new(repo, providers));
    let router = HttpApi::new(service, &config.frontend_dir, &config.public_url).router();

    let listener = tokio::net::TcpListener::bind(&config.addr)
        .await
        .with_context(|| format!("cannot listen on {}", config.addr))?;
    tracing::info!(addr = %listener.local_addr()?, frontend_dir = %config.frontend_dir.display(), "listening");
    axum::serve(listener, router).await.context("serving")
}
