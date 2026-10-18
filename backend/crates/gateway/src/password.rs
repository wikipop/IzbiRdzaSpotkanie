//! Optional nick passwords, hashed with Argon2id.

use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};

/// Argon2id with 7 MiB of memory and 5 iterations — one of the OWASP
/// recommended configurations, picked for the lowest memory use since RAM
/// is the scarcest resource of this deployment.
fn argon2() -> Argon2<'static> {
    let params = Params::new(7 * 1024, 5, 1, None).expect("static Argon2 parameters are valid");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

pub fn hash(password: &str) -> String {
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt).expect("the OS random source is available");
    let salt = SaltString::encode_b64(&salt).expect("16 bytes is a valid salt length");
    argon2()
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing with valid parameters cannot fail")
        .to_string()
}

/// Checks `password` against a PHC string produced by [`hash`]. The
/// parameters are read from the stored hash, so they can change later
/// without invalidating existing passwords.
pub fn verify(password: &str, stored: &str) -> bool {
    PasswordHash::new(stored).is_ok_and(|parsed| {
        argon2()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}
