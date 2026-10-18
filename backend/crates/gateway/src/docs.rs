use axum::Router;
use axum::routing::get;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Gateway",
        description = "Events, participants and tokens. Availability lives in the \
                       nano services, see `/docs` on any slot."
    ),
    paths(routes::create_event, routes::get_event, routes::join, routes::health),
    tags((name = "gateway", description = "Events and participants"))
)]
struct ApiDoc;

/// OpenAPI document of the gateway.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn docs_router() -> Router {
    let spec = openapi();
    Router::new()
        .route(
            "/openapi.json",
            get(move || async move { axum::Json(spec) }),
        )
        .merge(Scalar::with_url("/docs", openapi()))
}
