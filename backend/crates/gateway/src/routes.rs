use std::sync::MutexGuard;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use domain::{EventId, Nick, UserId};
use proto::{
    ApiError, ApiJson, CreateEventRequest, CreateEventResponse, ErrorCode, EventResponse,
    GatewayHealth, JoinRequest, JoinResponse,
};
use rusqlite::Connection;

use crate::db::{self, Participant};
use crate::{GatewayState, password, slots};

const MAX_EVENT_NAME_CHARS: usize = 100;
const MAX_PASSWORD_CHARS: usize = 128;

pub fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/health", get(health))
        .route("/events", post(create_event))
        .route("/events/{eid}", get(get_event))
        .route("/events/{eid}/join", post(join))
}

impl GatewayState {
    fn db(&self) -> MutexGuard<'_, Connection> {
        // With `panic = "abort"` a poisoned mutex cannot be observed in release.
        self.db.lock().expect("database mutex poisoned")
    }
}

fn db_error(error: rusqlite::Error) -> ApiError {
    tracing::error!(%error, "database error");
    ApiError::internal()
}

fn event_not_found() -> ApiError {
    ApiError::new(ErrorCode::EventNotFound, "event not found")
}

/// A malformed id cannot belong to any event, so it is reported as 404.
fn parse_event_id(raw: &str) -> Result<EventId, ApiError> {
    EventId::parse(raw).map_err(|_| event_not_found())
}

/// Create event
///
/// The slot range cannot be changed afterwards.
#[utoipa::path(
    post,
    path = "/events",
    tag = "gateway",
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Event created", body = CreateEventResponse),
        (status = 400, description = "Invalid name or slot range", body = proto::ErrorBody),
    )
)]
pub async fn create_event(
    State(state): State<GatewayState>,
    ApiJson(body): ApiJson<CreateEventRequest>,
) -> Result<(StatusCode, ApiJson<CreateEventResponse>), ApiError> {
    let name = body.name.trim();
    if name.is_empty() || name.chars().count() > MAX_EVENT_NAME_CHARS {
        return Err(ApiError::new(
            ErrorCode::InvalidEventName,
            "event name must be 1–100 characters long",
        ));
    }
    let slots = slots::resolve(&body)?;
    let event_id = EventId::generate();
    db::insert_event(&state.db(), &event_id, name, &slots).map_err(db_error)?;
    tracing::info!(%event_id, slots = slots.len(), "event created");
    Ok((
        StatusCode::CREATED,
        ApiJson(CreateEventResponse {
            event_id: event_id.to_string(),
        }),
    ))
}

/// Get event
///
/// Public metadata of an event.
#[utoipa::path(
    get,
    path = "/events/{eid}",
    tag = "gateway",
    params(("eid" = String, Path, description = "Event id (`e_…`)")),
    responses(
        (status = 200, description = "Event metadata", body = EventResponse),
        (status = 404, description = "No such event", body = proto::ErrorBody),
    )
)]
pub async fn get_event(
    State(state): State<GatewayState>,
    Path(eid): Path<String>,
) -> Result<ApiJson<EventResponse>, ApiError> {
    let event_id = parse_event_id(&eid)?;
    let event = db::find_event(&state.db(), &event_id)
        .map_err(db_error)?
        .ok_or_else(event_not_found)?;
    Ok(ApiJson(EventResponse {
        event_id: event_id.to_string(),
        name: event.name,
        slots: event.slots.iter().map(|slot| slot.to_string()).collect(),
    }))
}

/// Join event
///
/// - a new nick creates a participant (password-protected if one is given);
/// - an existing nick without a password logs in as that participant;
/// - an existing nick with a password requires that password.
#[utoipa::path(
    post,
    path = "/events/{eid}/join",
    tag = "gateway",
    params(("eid" = String, Path, description = "Event id (`e_…`)")),
    request_body = JoinRequest,
    responses(
        (status = 200, description = "Participant id and token", body = JoinResponse),
        (status = 400, description = "Invalid nick or password", body = proto::ErrorBody),
        (status = 401, description = "Wrong or missing password", body = proto::ErrorBody),
        (status = 404, description = "No such event", body = proto::ErrorBody),
    )
)]
pub async fn join(
    State(state): State<GatewayState>,
    Path(eid): Path<String>,
    ApiJson(body): ApiJson<JoinRequest>,
) -> Result<ApiJson<JoinResponse>, ApiError> {
    let event_id = parse_event_id(&eid)?;
    let nick = Nick::parse(&body.nick)
        .map_err(|e| ApiError::new(ErrorCode::InvalidNick, e.to_string()))?;
    let password = body.password.filter(|password| !password.is_empty());
    if password
        .as_ref()
        .is_some_and(|p| p.chars().count() > MAX_PASSWORD_CHARS)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidPassword,
            "password must be at most 128 characters long",
        ));
    }

    let event = db::find_event(&state.db(), &event_id)
        .map_err(db_error)?
        .ok_or_else(event_not_found)?;
    let existing = db::find_participant(&state.db(), &event_id, &nick.key()).map_err(db_error)?;

    let participant = match existing {
        Some(participant) => {
            authenticate(&participant, password).await?;
            participant
        }
        None => match register(&state, &event_id, &nick, password.clone()).await? {
            Some(participant) => participant,
            // A concurrent join registered the same nick first: log in instead.
            None => {
                let participant = db::find_participant(&state.db(), &event_id, &nick.key())
                    .map_err(db_error)?
                    .ok_or_else(ApiError::internal)?;
                authenticate(&participant, password).await?;
                participant
            }
        },
    };

    let token = state
        .issuer
        .issue(&participant.id, &participant.nick, &event_id, &event.slots);
    Ok(ApiJson(JoinResponse {
        user_id: participant.id.to_string(),
        token,
    }))
}

/// Creates a participant; returns `None` if the nick got taken meanwhile.
async fn register(
    state: &GatewayState,
    event_id: &EventId,
    nick: &Nick,
    password: Option<String>,
) -> Result<Option<Participant>, ApiError> {
    let password_hash = match password {
        Some(password) => Some(run_blocking(move || password::hash(&password)).await?),
        None => None,
    };
    let participant = Participant {
        id: UserId::generate(),
        nick: nick.to_string(),
        password_hash,
    };
    let inserted = db::insert_participant(&state.db(), event_id, &participant, &nick.key())
        .map_err(db_error)?;
    if inserted {
        tracing::info!(%event_id, user_id = %participant.id, "participant joined");
    }
    Ok(inserted.then_some(participant))
}

/// Checks the password of an existing participant. A participant without a
/// password accepts anyone using the nick (and ignores a supplied password).
async fn authenticate(participant: &Participant, password: Option<String>) -> Result<(), ApiError> {
    let Some(stored) = participant.password_hash.clone() else {
        return Ok(());
    };
    let invalid = || {
        ApiError::new(
            ErrorCode::InvalidCredentials,
            "this nick is password-protected",
        )
    };
    let password = password.ok_or_else(invalid)?;
    let valid = run_blocking(move || password::verify(&password, &stored)).await?;
    if valid { Ok(()) } else { Err(invalid()) }
}

/// Argon2 takes milliseconds of CPU, which would stall the single-threaded
/// runtime, so it runs on Tokio's blocking thread pool.
async fn run_blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work).await.map_err(|error| {
        tracing::error!(%error, "blocking task failed");
        ApiError::internal()
    })
}

/// Health check
#[utoipa::path(
    get,
    path = "/health",
    tag = "gateway",
    responses((status = 200, description = "Service and database are up", body = GatewayHealth))
)]
pub async fn health(State(state): State<GatewayState>) -> Result<ApiJson<GatewayHealth>, ApiError> {
    db::ping(&state.db()).map_err(db_error)?;
    Ok(ApiJson(GatewayHealth {
        status: "ok".to_owned(),
    }))
}
