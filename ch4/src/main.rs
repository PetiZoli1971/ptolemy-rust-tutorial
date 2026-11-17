//! Chapter 4: Error handling. `Option`, `Result`, `?`, custom error types and `thiserror`.

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use thiserror::Error;

// ---- what `thiserror` writes for you ---------------------------------------

#[derive(Debug, PartialEq, Eq)]
enum ManualJdError {
    BeforeEpoch,
}

impl fmt::Display for ManualJdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManualJdError::BeforeEpoch => {
                f.write_str("Julian Day is only defined from -4712-01-01 12:00:00")
            }
        }
    }
}

impl Error for ManualJdError {}

// ---- the real error types --------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Error)]
enum DateError {
    #[error("month must be between 1 and 12, got {0}")]
    Month(u8),
    #[error("day {day} is not valid for {year}-{month:02}")]
    Day { year: i32, month: u8, day: u8 },
    #[error("dates between 1582-10-05 and 1582-10-14 do not exist")]
    SkippedByReform,
    #[error("hour must be between 0 and 23, got {0}")]
    Hour(u8),
    #[error("minute must be between 0 and 59, got {0}")]
    Minute(u8),
    #[error("second must be between 0 and 59, got {0}")]
    Second(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
enum JdError {
    #[error("Julian Day is only defined from -4712-01-01 12:00:00")]
    BeforeEpoch,
}

fn is_leap_year(year: i32) -> bool {
    if year <= 1582 {
        year % 4 == 0
    } else {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }
}

fn days_in_month(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if is_leap_year(year) { 29 } else { 28 }),
        _ => None,
    }
}

// ---- a validated type: "parse, don't validate" -----------------------------

const SECS_PER_DAY: i64 = 86_400;

/// Fields are private: the only way to get a value is `new`, which validates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct HistoricDateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    sec: u8,
}

impl HistoricDateTime {
    fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        sec: u8,
    ) -> Result<Self, DateError> {
        // `ok_or` turns an `Option` into a `Result`; `?` returns early on `Err`.
        let max_day = days_in_month(year, month).ok_or(DateError::Month(month))?;
        if year == 1582 && month == 10 && (5..15).contains(&day) {
            return Err(DateError::SkippedByReform);
        }
        if !(1..=max_day).contains(&day) {
            return Err(DateError::Day { year, month, day });
        }
        if hour >= 24 {
            return Err(DateError::Hour(hour));
        }
        if minute >= 60 {
            return Err(DateError::Minute(minute));
        }
        if sec >= 60 {
            return Err(DateError::Second(sec));
        }
        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            sec,
        })
    }

    fn is_gregorian(&self) -> bool {
        (self.year, self.month, self.day) >= (1582, 10, 15)
    }

    /// Julian Day Number of the noon of this calendar day, in integer arithmetic.
    fn julian_day_number(&self) -> i64 {
        let (y, m) = if self.month <= 2 {
            (i64::from(self.year) - 1, i64::from(self.month) + 12)
        } else {
            (i64::from(self.year), i64::from(self.month))
        };
        let b = if self.is_gregorian() {
            let a = y.div_euclid(100);
            2 - a + a.div_euclid(4)
        } else {
            0
        };
        (1461 * (y + 4716)).div_euclid(4)
            + (306_001 * (m + 1)).div_euclid(10_000)
            + i64::from(self.day)
            + b
            - 1524
    }

    fn jd(&self) -> Result<f64, JdError> {
        let time_of_day =
            i64::from(self.hour) * 3_600 + i64::from(self.minute) * 60 + i64::from(self.sec);
        let secs = self.julian_day_number() * SECS_PER_DAY + time_of_day - SECS_PER_DAY / 2;
        if secs < 0 {
            Err(JdError::BeforeEpoch)
        } else {
            Ok(secs as f64 / SECS_PER_DAY as f64)
        }
    }
}

// ---- combining errors with `#[from]` ---------------------------------------

#[derive(Debug, Error, PartialEq, Eq)]
enum ParseError {
    #[error("expected YYYY.MM.DD, got {0:?}")]
    Format(String),
    #[error("not a number: {0}")]
    Number(#[from] ParseIntError),
    #[error(transparent)]
    Date(#[from] DateError),
}

/// Parses `YYYY.MM.DD` (the separator is `.` so that negative years work).
fn parse_date(s: &str) -> Result<HistoricDateTime, ParseError> {
    let mut parts = s.split('.');
    let (Some(y), Some(m), Some(d), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ParseError::Format(s.to_owned()));
    };
    // Each `?` converts the error with `From`: ParseIntError -> ParseError, DateError -> ParseError.
    Ok(HistoricDateTime::new(
        y.parse()?,
        m.parse()?,
        d.parse()?,
        0,
        0,
        0,
    )?)
}

#[derive(Debug, Error)]
enum AppError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Jd(#[from] JdError),
}

fn days_between(from: &str, to: &str) -> Result<f64, AppError> {
    let a = parse_date(from)?.jd()?;
    let b = parse_date(to)?.jd()?;
    Ok(b - a)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", ManualJdError::BeforeEpoch);
    println!("{}", JdError::BeforeEpoch);

    // 1. Matching on a Result.
    match HistoricDateTime::new(1582, 10, 10, 0, 0, 0) {
        Ok(d) => println!("valid: {d:?}"),
        Err(e) => println!("error: {e}"),
    }

    // 2. Inspecting variants and their data.
    let err = HistoricDateTime::new(1900, 2, 29, 0, 0, 0).unwrap_err();
    println!("{err}   ({err:?})");
    if let DateError::Day { year, .. } = err {
        println!("the offending year was {year}");
    }

    // 3. Fallbacks.
    let jd = HistoricDateTime::new(-4800, 1, 1, 0, 0, 0)?
        .jd()
        .unwrap_or(f64::NAN);
    println!("before the epoch: {jd}");

    // 4. Chained conversions with `?`.
    for text in [
        "2000.01.01",
        "2000.13.01",
        "20x0.01.01",
        "2000.01",
        "-4712.01.01",
    ] {
        match parse_date(text) {
            Ok(d) => println!("{text:<12} -> {d:?}, JD {:?}", d.jd()),
            Err(e) => println!("{text:<12} -> {e}"),
        }
    }

    println!(
        "days from 1957.10.04 to 2000.01.01: {:.1}",
        days_between("1957.10.04", "2000.01.01")?
    );
    println!(
        "{:?}",
        days_between("1957.10.04", "-5000.01.01").map_err(|e| e.to_string())
    );

    // 5. `main` may return an error: the program then exits with a non-zero code.
    let last = parse_date("2000.02.30")?;
    println!("not reached: {last:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_the_reform_gap() {
        for day in 5..=14 {
            assert_eq!(
                HistoricDateTime::new(1582, 10, day, 0, 0, 0),
                Err(DateError::SkippedByReform)
            );
        }
        assert!(HistoricDateTime::new(1582, 10, 15, 0, 0, 0).is_ok());
    }

    #[test]
    fn rejects_bad_fields() {
        assert_eq!(
            HistoricDateTime::new(100, 13, 1, 0, 0, 0),
            Err(DateError::Month(13))
        );
        assert_eq!(
            HistoricDateTime::new(100, 1, 1, 24, 0, 0),
            Err(DateError::Hour(24))
        );
        assert!(matches!(
            HistoricDateTime::new(1900, 2, 29, 0, 0, 0),
            Err(DateError::Day { .. })
        ));
    }

    #[test]
    fn julian_day_anchors() {
        let j2000 = HistoricDateTime::new(2000, 1, 1, 12, 0, 0).unwrap();
        assert_eq!(j2000.jd(), Ok(2_451_545.0));
        let epoch = HistoricDateTime::new(-4712, 1, 1, 12, 0, 0).unwrap();
        assert_eq!(epoch.jd(), Ok(0.0));
        let before = HistoricDateTime::new(-4712, 1, 1, 11, 59, 59).unwrap();
        assert_eq!(before.jd(), Err(JdError::BeforeEpoch));
    }

    #[test]
    fn parse_errors_are_converted() {
        assert!(matches!(
            parse_date("20x0.01.01"),
            Err(ParseError::Number(_))
        ));
        assert!(matches!(
            parse_date("2000.13.01"),
            Err(ParseError::Date(DateError::Month(13)))
        ));
        assert_eq!(
            parse_date("2000.01"),
            Err(ParseError::Format("2000.01".into()))
        );
        assert!(parse_date("-747.02.28").is_ok());
    }
}
