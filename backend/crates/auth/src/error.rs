use proto::{ApiError, ErrorCode};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthError {
    #[error("missing Authorization: Bearer header")]
    MissingToken,
    #[error("token is signed with an unknown key")]
    UnknownKey,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("token belongs to a different event")]
    EventMismatch,
    #[error("slot {0} is not part of the event")]
    SlotOutOfRange(String),
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        let code = match error {
            AuthError::MissingToken => ErrorCode::MissingToken,
            AuthError::UnknownKey | AuthError::InvalidToken(_) => ErrorCode::InvalidToken,
            AuthError::EventMismatch => ErrorCode::EventMismatch,
            AuthError::SlotOutOfRange(_) => ErrorCode::SlotOutOfRange,
        };
        ApiError::new(code, error.to_string())
    }
}
