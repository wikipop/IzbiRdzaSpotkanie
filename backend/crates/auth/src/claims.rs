use serde::{Deserialize, Serialize};

/// Token payload. It carries everything a nano service needs to authorize a
/// write, so the service never has to ask the gateway about the participant
/// or the event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer, `api-meet.izbi.beer` by default.
    pub iss: String,
    /// Participant id (`u_…`) — the only source of `user_id` for writes.
    pub sub: String,
    /// Nick in its original spelling, denormalized into nano service entries.
    pub nick: String,
    /// Event id (`e_…`).
    pub eid: String,
    /// Event slot range: a 168-bit bitmap in base64url.
    pub slots: String,
    /// Issued-at time, seconds since the Unix epoch.
    pub iat: u64,
    /// Expiry time, seconds since the Unix epoch.
    pub exp: u64,
}
