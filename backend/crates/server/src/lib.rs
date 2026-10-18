//! Plumbing shared by the `gateway`, `nano` and `monolith` binaries.
//!
//! Business logic lives in the `gateway` and `nano` libraries; this crate
//! only covers what every process needs around it, so the three binaries
//! stay a few lines long and behave identically.

mod config;
mod cors;
mod healthcheck;
mod shutdown;
mod telemetry;

pub use config::{env_or, optional_env, required_env};
pub use cors::{OriginPattern, cors_layer, parse_origins};
pub use healthcheck::healthcheck;
pub use shutdown::shutdown_signal;
pub use telemetry::init_tracing;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

/// Serves `app` on `0.0.0.0:{port}` with CORS and request tracing until
/// SIGTERM or Ctrl+C, then finishes in-flight requests and returns.
pub async fn serve(app: Router, port: u16, allowed_origins: &str) -> anyhow::Result<()> {
    let app = app
        .layer(cors_layer(parse_origins(allowed_origins)?))
        .layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("shut down gracefully");
    Ok(())
}
