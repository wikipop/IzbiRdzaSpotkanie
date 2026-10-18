//! `nano` binary: one process per hour of the week.
//!
//! `nano` serves the slot from `SLOT_ID`; `nano healthcheck` probes the
//! local `/health` endpoint (used by Docker's `HEALTHCHECK`).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use auth::{DEFAULT_ISSUER, TokenVerifier};
use domain::SlotId;
use server::{env_or, required_env};

const DEFAULT_PORT: u16 = 8080;

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

    let slot: SlotId = required_env("SLOT_ID")?
        .parse()
        .context("invalid SLOT_ID")?;
    let data_dir: PathBuf = env_or("DATA_DIR", PathBuf::from("/data"))?;
    let issuer = env_or("JWT_ISSUER", DEFAULT_ISSUER.to_owned())?;
    let verifier = TokenVerifier::from_env_spec(&required_env("JWT_PUBLIC_KEYS")?, &issuer)
        .context("invalid JWT_PUBLIC_KEYS")?;
    let allowed_origins = env_or("ALLOWED_ORIGIN", "http://localhost:*".to_owned())?;

    let db_path = data_dir.join(format!("{slot}.sqlite"));
    let db = nano::open_database(&db_path)
        .with_context(|| format!("cannot open {}", db_path.display()))?;

    tracing::info!(%slot, db = %db_path.display(), "starting nano service");
    let app = nano::router(slot, db, Arc::new(verifier));
    server::serve(app, port, &allowed_origins).await
}
