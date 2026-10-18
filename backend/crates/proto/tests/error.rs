use axum::http::StatusCode;
use axum::response::IntoResponse;
use http_body_util::BodyExt;
use proto::{ApiError, ErrorCode};

#[tokio::test]
async fn serializes_to_common_error_format() {
    let response = ApiError::new(ErrorCode::SlotOutOfRange, "out of range").into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "error": { "code": "slot_out_of_range", "message": "out of range" }
        })
    );
}
