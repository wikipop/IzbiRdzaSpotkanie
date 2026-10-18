//! `cargo run -p monolith` — the whole backend on one port for local work.
//!
//! Without `JWT_PRIVATE_KEY`/`JWT_PUBLIC_KEYS` it generates a throwaway key
//! pair, so tokens stop working after a restart (just join again).

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use auth::{DEFAULT_ISSUER, TokenIssuer, TokenVerifier, generate_keypair, unescape_pem};
use server::{env_or, optional_env};

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_TOKEN_TTL_DAYS: u64 = 14;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    server::init_tracing();

    let port = env_or("PORT", DEFAULT_PORT)?;
    let data_dir: PathBuf = env_or("DATA_DIR", PathBuf::from("./data"))?;
    let allowed_origins = env_or("ALLOWED_ORIGIN", "http://localhost:*".to_owned())?;
    let issuer_name = env_or("JWT_ISSUER", DEFAULT_ISSUER.to_owned())?;
    let ttl = Duration::from_secs(env_or("TOKEN_TTL_DAYS", DEFAULT_TOKEN_TTL_DAYS)? * 24 * 3600);

    let (issuer, verifier) = match (
        optional_env("JWT_PRIVATE_KEY"),
        optional_env("JWT_PUBLIC_KEYS"),
    ) {
        (Some(private_pem), Some(public_keys)) => (
            TokenIssuer::new(
                env_or("JWT_KEY_ID", "k1".to_owned())?,
                &unescape_pem(&private_pem),
                &issuer_name,
                ttl,
            )
            .context("invalid JWT_PRIVATE_KEY")?,
            TokenVerifier::from_env_spec(&public_keys, &issuer_name)
                .context("invalid JWT_PUBLIC_KEYS")?,
        ),
        _ => {
            tracing::warn!("JWT keys not configured, using an ephemeral key pair");
            let pair = generate_keypair();
            (
                TokenIssuer::new("dev", &pair.private_pem, &issuer_name, ttl)?,
                TokenVerifier::new(&[("dev".to_owned(), pair.public_pem)], &issuer_name)?,
            )
        }
    };

    let app = monolith::app(&data_dir, issuer, verifier)?;
    tracing::info!(
        data_dir = %data_dir.display(),
        "gateway at http://localhost:{port}/, slots at http://localhost:{port}/slots/{{slot}}/"
    );
    server::serve(app, port, &allowed_origins).await
}
