//! Authentication with Ed25519-signed (EdDSA) JWTs.
//!
//! - the gateway holds the private key and issues tokens ([`TokenIssuer`]);
//! - nano services only hold public keys and verify tokens locally
//!   ([`TokenVerifier`], the [`AuthUser`] extractor) — without ever calling
//!   the gateway.
//!
//! Decisions and trade-offs are described in ADR 0003.

mod claims;
mod error;
mod extractor;
mod issuer;
mod keys;
mod verifier;

pub use claims::Claims;
pub use error::AuthError;
pub use extractor::{AuthUser, SlotAuth, authorize};
pub use issuer::TokenIssuer;
pub use keys::{KeyError, KeyPair, generate_keypair, parse_public_keys, unescape_pem};
pub use verifier::TokenVerifier;

/// Default token issuer (the `iss` claim).
pub const DEFAULT_ISSUER: &str = "api-meet.izbi.beer";
