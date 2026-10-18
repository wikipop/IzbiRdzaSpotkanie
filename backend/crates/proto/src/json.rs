use axum::extract::FromRequest;
use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};

use crate::{ApiError, ErrorCode};

/// Drop-in replacement for `axum::Json` that reports parse failures in the
/// common error format instead of plain text.
#[derive(Debug, Clone, Copy, Default, FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::new(ErrorCode::InvalidBody, rejection.body_text())
    }
}

impl<T: serde::Serialize> IntoResponse for ApiJson<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
