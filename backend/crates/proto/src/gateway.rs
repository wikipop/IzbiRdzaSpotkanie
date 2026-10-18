use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Creates an event. The slot range is given **either** as a `slots` list
/// **or** as `days` + `hours` (their Cartesian product).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({ "name": "Club meeting", "days": ["pn", "wt", "sr"], "hours": { "from": 9, "to": 17 } }))]
pub struct CreateEventRequest {
    /// Event name, 1–100 characters after trimming.
    pub name: String,
    /// Explicit slot list, e.g. `["pn-09", "pn-10"]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slots: Option<Vec<String>>,
    /// Days of the week: `pn wt sr cz pt sb nd`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days: Option<Vec<String>>,
    /// Hour range applied to each of `days`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours: Option<HourRange>,
}

/// Half-open hour range `[from, to)`: `{ from: 9, to: 17 }` means slots 09–16,
/// i.e. a meeting can start at 16:00 at the latest and end at 17:00.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct HourRange {
    /// First hour, 0–23.
    pub from: u8,
    /// End hour (exclusive), 1–24, greater than `from`.
    pub to: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateEventResponse {
    #[schema(example = "e_Jm1qY2g2c3Q4cHVpd2E5eA")]
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventResponse {
    #[schema(example = "e_Jm1qY2g2c3Q4cHVpd2E5eA")]
    pub event_id: String,
    pub name: String,
    /// Event slots in index order (starting from Monday 00).
    #[schema(example = json!(["pn-09", "pn-10", "wt-09"]))]
    pub slots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JoinRequest {
    /// Nick, 1–32 characters after trimming; compared case-insensitively.
    pub nick: String,
    /// Optional password protecting the nick. An empty string means no password.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JoinResponse {
    #[schema(example = "u_bG9yZW1pcHN1bWRvbG9yc2k")]
    pub user_id: String,
    /// JWT for the `Authorization: Bearer …` header of writes to nano services.
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GatewayHealth {
    #[schema(example = "ok")]
    pub status: String,
}
