use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Machine-readable error code. The frontend should branch on the code and
/// treat `message` as a human-readable hint only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The request body is not valid JSON or has the wrong shape.
    InvalidBody,
    /// Invalid event name.
    InvalidEventName,
    /// Invalid slot id, day or hour range.
    InvalidSlot,
    /// Invalid nick.
    InvalidNick,
    /// Invalid password (e.g. too long).
    InvalidPassword,
    /// Malformed event identifier.
    InvalidEventId,
    /// The event does not exist.
    EventNotFound,
    /// Missing `Authorization: Bearer …` header.
    MissingToken,
    /// The token is invalid, expired or signed with an unknown key.
    InvalidToken,
    /// The nick is password-protected and the password is wrong or missing.
    InvalidCredentials,
    /// The token belongs to a different event than the one in the path.
    EventMismatch,
    /// The slot is not part of the event's range.
    SlotOutOfRange,
    /// Unexpected server error.
    Internal,
}

impl ErrorCode {
    pub fn status(self) -> StatusCode {
        match self {
            ErrorCode::InvalidBody
            | ErrorCode::InvalidEventName
            | ErrorCode::InvalidSlot
            | ErrorCode::InvalidNick
            | ErrorCode::InvalidPassword
            | ErrorCode::InvalidEventId => StatusCode::BAD_REQUEST,
            ErrorCode::EventNotFound => StatusCode::NOT_FOUND,
            ErrorCode::MissingToken | ErrorCode::InvalidToken | ErrorCode::InvalidCredentials => {
                StatusCode::UNAUTHORIZED
            }
            ErrorCode::EventMismatch | ErrorCode::SlotOutOfRange => StatusCode::FORBIDDEN,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Body of every error response: `{ "error": { "code": …, "message": … } }`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    #[schema(example = "slot pn-03 is not part of this event")]
    pub message: String,
}

/// Error returned by handlers. The HTTP status is derived from the code.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ApiError {
            code,
            message: message.into(),
        }
    }

    /// Internal error. Details go to the logs; the client gets a generic message.
    pub fn internal() -> Self {
        ApiError::new(ErrorCode::Internal, "internal server error")
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
            },
        };
        (self.code.status(), Json(body)).into_response()
    }
}
