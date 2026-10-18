//! Nano service: availability of all events in exactly one hour of the week.
//!
//! The same router is used by the standalone `nano` binary (one process per
//! slot) and by the `monolith` (168 routers in one process).

mod db;
mod docs;
mod routes;

use std::sync::{Arc, Mutex};

use auth::{SlotAuth, TokenVerifier};
use axum::Router;
use axum::extract::FromRef;
use domain::SlotId;
use rusqlite::Connection;

pub use db::open_database;
pub use docs::openapi;

/// Shared state of one nano service.
#[derive(Clone)]
pub struct NanoState {
    slot: SlotId,
    db: Arc<Mutex<Connection>>,
    auth: SlotAuth,
}

impl FromRef<NanoState> for SlotAuth {
    fn from_ref(state: &NanoState) -> Self {
        state.auth.clone()
    }
}

/// Builds the router of the nano service for `slot`.
///
/// `db` is the slot's own SQLite connection (see [`open_database`]); a
/// single connection behind a mutex is enough because every query takes
/// microseconds (ADR 0001).
pub fn router(slot: SlotId, db: Connection, verifier: Arc<TokenVerifier>) -> Router {
    let state = NanoState {
        slot,
        db: Arc::new(Mutex::new(db)),
        auth: SlotAuth { verifier, slot },
    };
    routes::routes()
        .with_state(state)
        .merge(docs::docs_router())
}
