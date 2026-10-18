use std::sync::MutexGuard;

use auth::AuthUser;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use domain::EventId;
use proto::{
    ApiError, ApiJson, AvailabilityEntry, ErrorCode, NanoHealth, PutAvailabilityRequest,
    SlotEntriesResponse,
};
use rusqlite::Connection;

use crate::{NanoState, db};

pub fn routes() -> Router<NanoState> {
    Router::new()
        .route("/health", get(health))
        .route("/events/{eid}", get(list_entries))
        .route("/events/{eid}/me", put(put_me).delete(delete_me))
}

impl NanoState {
    fn db(&self) -> MutexGuard<'_, Connection> {
        // With `panic = "abort"` a poisoned mutex cannot be observed in release.
        self.db.lock().expect("database mutex poisoned")
    }
}

fn db_error(error: rusqlite::Error) -> ApiError {
    tracing::error!(%error, "database error");
    ApiError::internal()
}

/// List availability
///
/// Everyone's availability in this slot. No auth, so the browser sends no
/// CORS preflight.
#[utoipa::path(
    get,
    path = "/events/{eid}",
    tag = "nano",
    params(("eid" = String, Path, description = "Event id (`e_…`)")),
    responses(
        (status = 200, description = "Entries of this slot", body = SlotEntriesResponse),
        (status = 400, description = "Malformed event id", body = proto::ErrorBody),
    )
)]
pub async fn list_entries(
    State(state): State<NanoState>,
    Path(eid): Path<String>,
) -> Result<ApiJson<SlotEntriesResponse>, ApiError> {
    let event_id = EventId::parse(&eid)
        .map_err(|e| ApiError::new(ErrorCode::InvalidEventId, e.to_string()))?;
    let entries = db::list_entries(&state.db(), event_id.as_str()).map_err(db_error)?;
    Ok(ApiJson(SlotEntriesResponse {
        slot: state.slot.to_string(),
        entries,
    }))
}

/// Set my availability
///
/// Inserts or updates the caller's entry in this slot.
#[utoipa::path(
    put,
    path = "/events/{eid}/me",
    tag = "nano",
    security(("bearer" = [])),
    params(("eid" = String, Path, description = "Event id (`e_…`)")),
    request_body = PutAvailabilityRequest,
    responses(
        (status = 200, description = "Stored entry", body = AvailabilityEntry),
        (status = 400, description = "Invalid body", body = proto::ErrorBody),
        (status = 401, description = "Missing or invalid token", body = proto::ErrorBody),
        (status = 403, description = "Token for another event or slot outside the event", body = proto::ErrorBody),
    )
)]
pub async fn put_me(
    State(state): State<NanoState>,
    user: AuthUser,
    ApiJson(body): ApiJson<PutAvailabilityRequest>,
) -> Result<ApiJson<AvailabilityEntry>, ApiError> {
    let entry = db::upsert_entry(
        &state.db(),
        &user.event_id,
        &user.user_id,
        &user.nick,
        body.state.into(),
    )
    .map_err(db_error)?;
    Ok(ApiJson(entry))
}

/// Clear my availability
///
/// Removes the caller's entry from this slot. Idempotent.
#[utoipa::path(
    delete,
    path = "/events/{eid}/me",
    tag = "nano",
    security(("bearer" = [])),
    params(("eid" = String, Path, description = "Event id (`e_…`)")),
    responses(
        (status = 204, description = "Entry removed (or there was none)"),
        (status = 401, description = "Missing or invalid token", body = proto::ErrorBody),
        (status = 403, description = "Token for another event or slot outside the event", body = proto::ErrorBody),
    )
)]
pub async fn delete_me(
    State(state): State<NanoState>,
    user: AuthUser,
) -> Result<StatusCode, ApiError> {
    db::delete_entry(&state.db(), &user.event_id, &user.user_id).map_err(db_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Health check
#[utoipa::path(
    get,
    path = "/health",
    tag = "nano",
    responses((status = 200, description = "Service and database are up", body = NanoHealth))
)]
pub async fn health(State(state): State<NanoState>) -> Result<ApiJson<NanoHealth>, ApiError> {
    db::ping(&state.db()).map_err(db_error)?;
    Ok(ApiJson(NanoHealth {
        slot: state.slot.to_string(),
        status: "ok".to_owned(),
    }))
}
