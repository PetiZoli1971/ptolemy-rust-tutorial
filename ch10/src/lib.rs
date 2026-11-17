//! Chapter 10: Testing. A tiny library whose only purpose is to be tested in every way Rust offers.
//!
//! The crate-level docs are tested too:
//!
//! ```
//! use ch10::julian_day;
//! assert_eq!(julian_day(2000, 1, 1, 12, 0, 0), 2_451_545.0);
//! ```

const SECS_PER_DAY: i64 = 86_400;

/// Julian rule up to and including 1582, Gregorian rule afterwards.
///
/// ```
/// use ch10::is_leap_year;
/// assert!(is_leap_year(2000));
/// assert!(!is_leap_year(1900));
/// assert!(is_leap_year(1500)); // Julian calendar
/// ```
pub fn is_leap_year(year: i32) -> bool {
    if year <= 1582 {
        year % 4 == 0
    } else {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }
}

/// Julian Day Number of the noon of a calendar day, in integer arithmetic.
///
/// # Panics
///
/// Panics if `month` is not in `1..=12`.
///
/// ```
/// assert_eq!(ch10::julian_day_number(2000, 1, 1), 2_451_545);
/// ```
///
/// ```should_panic
/// ch10::julian_day_number(2000, 13, 1);
/// ```
pub fn julian_day_number(year: i32, month: u8, day: u8) -> i64 {
    assert!(
        (1..=12).contains(&month),
        "month must be between 1 and 12, got {month}"
    );
    let (y, m) = if month <= 2 {
        (i64::from(year) - 1, i64::from(month) + 12)
    } else {
        (i64::from(year), i64::from(month))
    };
    let gregorian = (year, month, day) >= (1582, 10, 15);
    let b = if gregorian {
        let a = y.div_euclid(100);
        2 - a + a.div_euclid(4)
    } else {
        0
    };
    (1461 * (y + 4716)).div_euclid(4) + (306_001 * (m + 1)).div_euclid(10_000) + i64::from(day) + b
        - 1524
}

/// Decimal Julian Day.
pub fn julian_day(year: i32, month: u8, day: u8, hour: u8, minute: u8, sec: u8) -> f64 {
    let time_of_day = i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(sec);
    let secs = julian_day_number(year, month, day) * SECS_PER_DAY + time_of_day - SECS_PER_DAY / 2;
    secs as f64 / SECS_PER_DAY as f64
}

/// Maps any angle into `[0, 360)`.
///
/// ```
/// assert_eq!(ch10::normalize_deg(-90.0), 270.0);
/// ```
pub fn normalize_deg(angle: f64) -> f64 {
    let wrapped = angle.rem_euclid(360.0);
    if wrapped >= 360.0 {
        0.0
    } else {
        wrapped
    }
}

/// Signed shortest difference `a - b` in degrees, in `[-180, 180)`.
pub fn angle_diff(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

// Unit tests live next to the code and may touch private items.
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn leap_years() {
        assert!(is_leap_year(-4));
        assert!(!is_leap_year(1700));
    }

    #[test]
    fn julian_day_of_j2000() {
        assert_abs_diff_eq!(
            julian_day(2000, 1, 1, 12, 0, 0),
            2_451_545.0,
            epsilon = 1e-9
        );
    }

    #[test]
    #[should_panic(expected = "month must be between 1 and 12")]
    fn invalid_month_panics() {
        julian_day_number(2000, 0, 1);
    }

    // A test may return `Result`, so `?` works inside it.
    #[test]
    fn returns_result() -> Result<(), String> {
        let n: i64 = "2451545".parse::<i64>().map_err(|e| e.to_string())?;
        assert_eq!(julian_day_number(2000, 1, 1), n);
        Ok(())
    }

    #[test]
    #[ignore = "slow in debug builds; run with `cargo test -- --ignored`"]
    fn consecutive_days_for_a_long_range() {
        let mut previous = julian_day_number(-4000, 3, 1);
        for year in -3999..2000 {
            let n = julian_day_number(year, 3, 1);
            assert!(n > previous);
            previous = n;
        }
    }
}
