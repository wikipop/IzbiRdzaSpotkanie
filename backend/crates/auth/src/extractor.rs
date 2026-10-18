use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts, Path};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use domain::{SlotBitmap, SlotId};
use proto::ApiError;

use crate::{AuthError, Claims, TokenVerifier};

/// Authorization context of a nano service: the token verifier and the slot
/// the service owns. The router state must expose it through `FromRef`.
#[derive(Debug, Clone)]
pub struct SlotAuth {
    pub verifier: Arc<TokenVerifier>,
    pub slot: SlotId,
}

/// An authenticated participant allowed to write to this slot of the event
/// from the `{eid}` path parameter.
///
/// The extractor requires the route to have an `{eid}` parameter. `user_id`
/// comes exclusively from the `sub` claim — never from the request body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    pub user_id: String,
    pub nick: String,
    pub event_id: String,
}

/// Checks whether verified claims allow writing to `slot` of event
/// `path_eid`. Kept separate from the extractor so it can be tested
/// without HTTP.
pub fn authorize(claims: Claims, path_eid: &str, slot: SlotId) -> Result<AuthUser, AuthError> {
    if claims.eid != path_eid {
        return Err(AuthError::EventMismatch);
    }
    let slots = SlotBitmap::from_base64url(&claims.slots)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
    if !slots.contains(slot) {
        return Err(AuthError::SlotOutOfRange(slot.to_string()));
    }
    Ok(AuthUser {
        user_id: claims.sub,
        nick: claims.nick,
        event_id: claims.eid,
    })
}

fn bearer_token(parts: &Parts) -> Result<&str, AuthError> {
    let value = parts
        .headers
        .get(AUTHORIZATION)
        .ok_or(AuthError::MissingToken)?
        .to_str()
        .map_err(|_| AuthError::MissingToken)?;
    value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or(AuthError::MissingToken)
}

impl<S> FromRequestParts<S> for AuthUser
where
    SlotAuth: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = SlotAuth::from_ref(state);
        let claims = auth.verifier.verify(bearer_token(parts)?)?;

        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::internal())?;
        let path_eid = params.get("eid").ok_or_else(ApiError::internal)?;

        Ok(authorize(claims, path_eid, auth.slot)?)
    }
}
