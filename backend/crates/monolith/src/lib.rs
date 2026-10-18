//! The whole system in one process: the gateway at `/` and the nano service
//! of slot `X` at `/slots/X/…`.
//!
//! It mounts exactly the same routers as the standalone binaries, each slot
//! with its own SQLite file, so behaviour matches production except for the
//! base URLs (ADR 0004).

use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use auth::{TokenIssuer, TokenVerifier};
use axum::Router;
use domain::SlotId;

/// Builds the combined router, creating `data_dir` and the database files
/// (`gateway.sqlite` and one `{slot}.sqlite` per slot) if needed.
pub fn app(
    data_dir: &Path,
    issuer: TokenIssuer,
    verifier: TokenVerifier,
) -> anyhow::Result<Router> {
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("cannot create {}", data_dir.display()))?;

    let gateway_db = gateway::open_database(&data_dir.join("gateway.sqlite"))
        .context("cannot open the gateway database")?;
    let mut app = gateway::router(gateway_db, issuer);

    let verifier = Arc::new(verifier);
    for slot in SlotId::all() {
        let db = nano::open_database(&data_dir.join(format!("{slot}.sqlite")))
            .with_context(|| format!("cannot open the database of slot {slot}"))?;
        app = app.nest(
            &format!("/slots/{slot}"),
            nano::router(slot, db, Arc::clone(&verifier)),
        );
    }
    Ok(app)
}
