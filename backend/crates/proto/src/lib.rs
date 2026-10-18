//! HTTP contract shared by the gateway, the nano services and the tests.
//!
//! Every type that crosses the network is defined here together with its
//! OpenAPI schema — the frontend sees exactly these structures.

mod error;
mod gateway;
mod json;
mod nano;

pub use error::{ApiError, ErrorBody, ErrorCode, ErrorDetail};
pub use gateway::{
    CreateEventRequest, CreateEventResponse, EventResponse, GatewayHealth, HourRange, JoinRequest,
    JoinResponse,
};
pub use json::ApiJson;
pub use nano::{AvailabilityEntry, NanoHealth, PutAvailabilityRequest, SlotEntriesResponse, State};
