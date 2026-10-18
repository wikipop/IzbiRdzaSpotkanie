use std::fmt;
use std::str::FromStr;

/// Declared availability in a slot. No entry means not available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Yes,
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown state `{0}`, expected `yes` or `maybe`")]
pub struct StateError(String);

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Yes => "yes",
            State::Maybe => "maybe",
        }
    }
}

impl FromStr for State {
    type Err = StateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yes" => Ok(State::Yes),
            "maybe" => Ok(State::Maybe),
            other => Err(StateError(other.to_owned())),
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
