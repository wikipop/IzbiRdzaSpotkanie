use std::fmt;

/// Maximum nick length in characters (not bytes).
const MAX_CHARS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NickError {
    #[error("nick must not be empty")]
    Empty,
    #[error("nick must be at most 32 characters long")]
    TooLong,
    #[error("nick must not contain control characters")]
    ControlCharacter,
}

/// Participant nick: whitespace-trimmed, 1–32 characters.
///
/// Nicks are compared case-insensitively through [`Nick::key`]; the original
/// spelling is kept for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nick(String);

impl Nick {
    pub fn parse(raw: &str) -> Result<Self, NickError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(NickError::Empty);
        }
        if trimmed.chars().count() > MAX_CHARS {
            return Err(NickError::TooLong);
        }
        if trimmed.chars().any(char::is_control) {
            return Err(NickError::ControlCharacter);
        }
        Ok(Nick(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Case-insensitive comparison key (full Unicode lowercasing, so
    /// `Łukasz` and `łukasz` are the same nick).
    pub fn key(&self) -> String {
        self.0.to_lowercase()
    }
}

impl fmt::Display for Nick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
