use std::time::Duration;

use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

/// One entry of `ALLOWED_ORIGIN`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginPattern {
    /// Exact origin, e.g. `https://meet.izbi.beer`.
    Exact(String),
    /// Origin with any port, written as `http://localhost:*`; stores the
    /// part before the port, e.g. `http://localhost:`.
    AnyPort(String),
}

impl OriginPattern {
    pub fn matches(&self, origin: &str) -> bool {
        match self {
            OriginPattern::Exact(exact) => origin == exact,
            OriginPattern::AnyPort(prefix) => origin
                .strip_prefix(prefix.as_str())
                .is_some_and(|port| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit())),
        }
    }
}

/// Parses `ALLOWED_ORIGIN`: a comma-separated list of origins, where an
/// entry ending in `:*` allows any port (handy for `http://localhost:*`).
pub fn parse_origins(spec: &str) -> anyhow::Result<Vec<OriginPattern>> {
    let patterns: Vec<OriginPattern> = spec
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| match entry.strip_suffix('*') {
            Some(prefix) if prefix.ends_with(':') => OriginPattern::AnyPort(prefix.to_owned()),
            _ => OriginPattern::Exact(entry.trim_end_matches('/').to_owned()),
        })
        .collect();
    anyhow::ensure!(
        !patterns.is_empty(),
        "ALLOWED_ORIGIN must list at least one origin"
    );
    Ok(patterns)
}

/// CORS policy shared by every service.
///
/// `max_age` is two hours so that browsers cache the preflight of
/// authenticated `PUT`/`DELETE` requests instead of repeating it for every
/// slot. Public `GET`s are "simple" requests and never trigger a preflight.
pub fn cors_layer(patterns: Vec<OriginPattern>) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
            origin
                .to_str()
                .is_ok_and(|origin| patterns.iter().any(|p| p.matches(origin)))
        }))
        .allow_methods([Method::GET, Method::PUT, Method::DELETE, Method::POST])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .max_age(Duration::from_secs(7200))
}
