//! Gateway: creates events, lets participants join and issues their tokens.
//!
//! It knows only events and participants. Everything about a particular
//! hour lives in that hour's nano service, and the gateway never talks to
//! any of them.

mod db;
mod docs;
mod password;
mod routes;
mod slots;

use std::sync::{Arc, Mutex};

use auth::TokenIssuer;
use axum::Router;
use rusqlite::Connection;

pub use db::open_database;
pub use docs::openapi;

/// Shared state of the gateway.
#[derive(Clone)]
pub struct GatewayState {
    db: Arc<Mutex<Connection>>,
    issuer: Arc<TokenIssuer>,
}

/// Builds the gateway router. `db` comes from [`open_database`].
pub fn router(db: Connection, issuer: TokenIssuer) -> Router {
    let state = GatewayState {
        db: Arc::new(Mutex::new(db)),
        issuer: Arc::new(issuer),
    };
    routes::routes()
        .with_state(state)
        .merge(docs::docs_router())
}
