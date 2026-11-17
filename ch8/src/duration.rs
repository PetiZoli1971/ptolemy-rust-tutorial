//! A signed span of time in 365-day years.

use std::fmt;

use crate::units::{SECS_PER_DAY, SECS_PER_HOUR, SECS_PER_MINUTE, SECS_PER_YEAR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

/// The field is private to this module: outside code must use the constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    total_secs: i64,
}

impl Duration {
    /// ```
    /// use ch8::{Duration, Sign};
    /// let d = Duration::new(Sign::Minus, 1, 2, 0, 0, 0);
    /// assert_eq!(d.total_secs(), -(365 + 2) * 86_400);
    /// ```
    pub fn new(sign: Sign, years: u32, days: u32, hours: u32, minutes: u32, secs: u32) -> Self {
        let magnitude = i64::from(years) * SECS_PER_YEAR
            + i64::from(days) * SECS_PER_DAY
            + i64::from(hours) * SECS_PER_HOUR
            + i64::from(minutes) * SECS_PER_MINUTE
            + i64::from(secs);
        Self {
            total_secs: match sign {
                Sign::Plus => magnitude,
                Sign::Minus => -magnitude,
            },
        }
    }

    pub fn total_secs(&self) -> i64 {
        self.total_secs
    }

    pub fn years(&self) -> u64 {
        self.total_secs.unsigned_abs() / SECS_PER_YEAR as u64
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.total_secs < 0 { '-' } else { '+' };
        write!(f, "{sign}{} years", self.years())
    }
}
