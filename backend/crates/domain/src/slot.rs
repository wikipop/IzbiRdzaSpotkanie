use std::fmt;
use std::str::FromStr;

/// Day of the week; the week starts on Monday.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Day {
    Pn,
    Wt,
    Sr,
    Cz,
    Pt,
    Sb,
    Nd,
}

impl Day {
    pub const ALL: [Day; 7] = [
        Day::Pn,
        Day::Wt,
        Day::Sr,
        Day::Cz,
        Day::Pt,
        Day::Sb,
        Day::Nd,
    ];

    /// Day number: 0 = Monday, 6 = Sunday.
    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Day::Pn => "pn",
            Day::Wt => "wt",
            Day::Sr => "sr",
            Day::Cz => "cz",
            Day::Pt => "pt",
            Day::Sb => "sb",
            Day::Nd => "nd",
        }
    }
}

impl FromStr for Day {
    type Err = SlotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Day::ALL
            .into_iter()
            .find(|day| day.as_str() == s)
            .ok_or_else(|| SlotError::InvalidDay(s.to_owned()))
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SlotError {
    #[error("unknown day `{0}`, expected one of: pn wt sr cz pt sb nd")]
    InvalidDay(String),
    #[error("invalid hour `{0}`, expected two digits 00–23")]
    InvalidHour(String),
    #[error("invalid slot id `{0}`, expected the `pn-09` format")]
    InvalidFormat(String),
    #[error("slot index {0} out of range 0..=167")]
    IndexOutOfRange(usize),
}

/// One hour of the week, e.g. `sr-09` (Wednesday 9:00–10:00).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SlotId {
    day: Day,
    hour: u8,
}

impl SlotId {
    /// Number of slots in a week: 7 days × 24 hours.
    pub const COUNT: usize = 7 * 24;

    pub fn new(day: Day, hour: u8) -> Result<Self, SlotError> {
        if hour > 23 {
            return Err(SlotError::InvalidHour(hour.to_string()));
        }
        Ok(SlotId { day, hour })
    }

    pub fn day(self) -> Day {
        self.day
    }

    pub fn hour(self) -> u8 {
        self.hour
    }

    /// Slot index `day * 24 + hour` (0..=167) — the same order as in the bitmap.
    pub fn index(self) -> usize {
        usize::from(self.day.index()) * 24 + usize::from(self.hour)
    }

    pub fn from_index(index: usize) -> Result<Self, SlotError> {
        if index >= Self::COUNT {
            return Err(SlotError::IndexOutOfRange(index));
        }
        let day = Day::ALL[index / 24];
        let hour = u8::try_from(index % 24).expect("index % 24 fits in u8");
        Ok(SlotId { day, hour })
    }

    /// All 168 slots in index order.
    pub fn all() -> impl Iterator<Item = SlotId> {
        (0..Self::COUNT).map(|index| Self::from_index(index).expect("index is in range"))
    }
}

impl FromStr for SlotId {
    type Err = SlotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (day, hour) = s
            .split_once('-')
            .ok_or_else(|| SlotError::InvalidFormat(s.to_owned()))?;
        let day: Day = day.parse()?;
        let is_two_digits = hour.len() == 2 && hour.bytes().all(|b| b.is_ascii_digit());
        if !is_two_digits {
            return Err(SlotError::InvalidHour(hour.to_owned()));
        }
        let hour: u8 = hour
            .parse()
            .map_err(|_| SlotError::InvalidHour(hour.to_owned()))?;
        SlotId::new(day, hour)
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:02}", self.day, self.hour)
    }
}
