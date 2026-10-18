use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeyError {
    #[error("empty public key list")]
    Empty,
    #[error("entry `{0}` is not in the kid=PEM format")]
    MissingKid(String),
    #[error("key `{0}` is not a valid Ed25519 PEM key")]
    InvalidPem(String),
    #[error("key `{0}` is listed more than once")]
    DuplicateKid(String),
}

/// Ed25519 key pair in PEM (PKCS#8 for the private key, SPKI for the public key).
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub private_pem: String,
    pub public_pem: String,
}

pub fn generate_keypair() -> KeyPair {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).expect("the OS random source is available");
    let signing_key = SigningKey::from_bytes(&seed);
    let private_pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("encoding an Ed25519 key as PKCS#8 cannot fail");
    let public_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .expect("encoding an Ed25519 key as SPKI cannot fail");
    KeyPair {
        private_pem: private_pem.to_string(),
        public_pem,
    }
}

/// Turns `\n` escape sequences into newlines so that a PEM can be passed as
/// a single-line environment variable (`.env`, the Dokploy panel).
pub fn unescape_pem(value: &str) -> String {
    value.replace("\\n", "\n")
}

/// Parses `JWT_PUBLIC_KEYS`: a comma-separated list of `kid=PEM` entries,
/// e.g. `k2=-----BEGIN PUBLIC KEY-----\n…,k1=-----BEGIN…`.
///
/// A PEM never contains a comma and a `kid` must not contain `=`, so the
/// split is unambiguous. Returns `(kid, pem)` pairs in the given order.
pub fn parse_public_keys(spec: &str) -> Result<Vec<(String, String)>, KeyError> {
    let mut keys: Vec<(String, String)> = Vec::new();
    for entry in spec.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        let (kid, pem) = entry
            .split_once('=')
            .ok_or_else(|| KeyError::MissingKid(entry.chars().take(20).collect()))?;
        let kid = kid.trim().to_owned();
        if kid.is_empty() {
            return Err(KeyError::MissingKid(entry.chars().take(20).collect()));
        }
        if keys.iter().any(|(existing, _)| *existing == kid) {
            return Err(KeyError::DuplicateKid(kid));
        }
        keys.push((kid, unescape_pem(pem.trim())));
    }
    if keys.is_empty() {
        return Err(KeyError::Empty);
    }
    Ok(keys)
}
