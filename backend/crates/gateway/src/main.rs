//! `gateway` binary: `api-meet.izbi.beer`.
//!
//! `gateway` serves the API; `gateway healthcheck` probes the local
//! `/health` endpoint (used by Docker's `HEALTHCHECK`).

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use auth::{DEFAULT_ISSUER, TokenIssuer, unescape_pem};
use server::{env_or, required_env};

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_TOKEN_TTL_DAYS: u64 = 14;

fn main() -> anyhow::Result<()> {
    let port = env_or("PORT", DEFAULT_PORT)?;
    match std::env::args().nth(1).as_deref() {
        Some("healthcheck") => server::healthcheck(port),
        Some(other) => anyhow::bail!("unknown subcommand `{other}`, expected `healthcheck`"),
        None => run(port),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn run(port: u16) -> anyhow::Result<()> {
    server::init_tracing();

    let data_dir: PathBuf = env_or("DATA_DIR", PathBuf::from("/data"))?;
    let ttl_days: u64 = env_or("TOKEN_TTL_DAYS", DEFAULT_TOKEN_TTL_DAYS)?;
    let issuer = TokenIssuer::new(
        env_or("JWT_KEY_ID", "k1".to_owned())?,
        &unescape_pem(&required_env("JWT_PRIVATE_KEY")?),
        env_or("JWT_ISSUER", DEFAULT_ISSUER.to_owned())?,
        Duration::from_secs(ttl_days * 24 * 3600),
    )
    .context("invalid JWT_PRIVATE_KEY")?;
    let allowed_origins = env_or("ALLOWED_ORIGIN", "http://localhost:*".to_owned())?;

    let db_path = data_dir.join("gateway.sqlite");
    let db = gateway::open_database(&db_path)
        .with_context(|| format!("cannot open {}", db_path.display()))?;

    tracing::info!(db = %db_path.display(), "starting gateway");
    let app = gateway::router(db, issuer);
    server::serve(app, port, &allowed_origins).await
}
