use tracing_subscriber::EnvFilter;

/// Initializes logging. `LOG_FORMAT=json` (set in the Docker images) emits
/// one JSON object per line; anything else prints human-readable logs.
/// The level is controlled with `RUST_LOG` and defaults to `info`.
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if std::env::var("LOG_FORMAT").is_ok_and(|format| format == "json") {
        builder.json().init();
    } else {
        builder.init();
    }
}
