use axum::Router;
use axum::routing::get;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_scalar::{Scalar, Servable};

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Nano",
        description = "Availability for one hour of the week. Each slot has its own \
                       base URL: `https://{slot}-meet.izbi.beer` in production, \
                       `http://localhost:8080/slots/{slot}` in the monolith."
    ),
    paths(routes::list_entries, routes::put_me, routes::delete_me, routes::health),
    modifiers(&BearerAuth),
    tags((name = "nano", description = "Per-slot availability"))
)]
struct ApiDoc;

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// OpenAPI document of the nano service.
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
