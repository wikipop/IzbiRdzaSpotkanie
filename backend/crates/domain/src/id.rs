use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// 128 random bits encoded as unpadded base64url take 22 characters.
const ENCODED_LEN: usize = 22;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid identifier `{0}`")]
pub struct IdError(String);

fn generate(prefix: &str) -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the OS random source is available");
    format!("{prefix}{}", URL_SAFE_NO_PAD.encode(bytes))
}

fn validate(prefix: &str, raw: &str) -> Result<(), IdError> {
    let valid = raw.strip_prefix(prefix).is_some_and(|rest| {
        rest.len() == ENCODED_LEN
            && rest
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    });
    if valid {
        Ok(())
    } else {
        Err(IdError(raw.to_owned()))
    }
}

macro_rules! prefixed_id {
    ($(#[$doc:meta])* $name:ident, $prefix:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn generate() -> Self {
                $name(generate(Self::PREFIX))
            }

            pub fn parse(raw: &str) -> Result<Self, IdError> {
                validate(Self::PREFIX, raw)?;
                Ok($name(raw.to_owned()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

prefixed_id!(
    /// Event identifier, e.g. `e_3q2-7wEXAMPLEidAAAAAA`. It cannot be guessed,
    /// so it works as a capability URL (see ADR 0002).
    EventId,
    "e_"
);

prefixed_id!(
    /// Event participant identifier, e.g. `u_…`.
    UserId,
    "u_"
);
