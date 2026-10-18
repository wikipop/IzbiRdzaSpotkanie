use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Availability state in a slot. No entry means not available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Yes,
    Maybe,
}

impl From<domain::State> for State {
    fn from(state: domain::State) -> Self {
        match state {
            domain::State::Yes => State::Yes,
            domain::State::Maybe => State::Maybe,
        }
    }
}

impl From<State> for domain::State {
    fn from(state: State) -> Self {
        match state {
            State::Yes => domain::State::Yes,
            State::Maybe => domain::State::Maybe,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PutAvailabilityRequest {
    pub state: State,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AvailabilityEntry {
    #[schema(example = "u_bG9yZW1pcHN1bWRvbG9yc2k")]
    pub user_id: String,
    #[schema(example = "Wiktor")]
    pub nick: String,
    pub state: State,
    /// Time of the last change, seconds since the Unix epoch.
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SlotEntriesResponse {
    #[schema(example = "sr-09")]
    pub slot: String,
    pub entries: Vec<AvailabilityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NanoHealth {
    #[schema(example = "sr-09")]
    pub slot: String,
    #[schema(example = "ok")]
    pub status: String,
}
