use anyhow::{Context, anyhow};

/// Reads a required environment variable.
pub fn required_env(name: &str) -> anyhow::Result<String> {
    std::env::var(name).map_err(|_| anyhow!("missing required environment variable {name}"))
}

/// Reads an optional environment variable; empty values count as missing.
pub fn optional_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

/// Reads and parses an environment variable, falling back to `default`.
pub fn env_or<T>(name: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match optional_env(name) {
        Some(value) => value
            .parse()
            .with_context(|| format!("invalid value of {name}: `{value}`")),
        None => Ok(default),
    }
}
