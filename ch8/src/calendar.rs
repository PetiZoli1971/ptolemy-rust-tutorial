//! Leap-year rule and month tables for the Julian/Gregorian hybrid calendar.

const MONTH_LENGTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Julian rule up to and including 1582, Gregorian rule afterwards.
///
/// ```
/// use ch8::calendar::is_leap_year;
/// assert!(is_leap_year(1500));
/// assert!(!is_leap_year(1900));
/// ```
pub fn is_leap_year(year: i32) -> bool {
    if year <= 1582 {
        year % 4 == 0
    } else {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }
}

pub fn days_in_month(year: i32, month: u8) -> Option<u8> {
    if !(1..=12).contains(&month) {
        return None;
    }
    if month == 2 && is_leap_year(year) {
        return Some(29);
    }
    Some(MONTH_LENGTHS[usize::from(month) - 1])
}

/// Days in a whole year; a private helper is only visible inside `calendar`.
pub fn days_in_year(year: i32) -> u16 {
    year_length(is_leap_year(year))
}

fn year_length(leap: bool) -> u16 {
    if leap {
        366
    } else {
        365
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_lengths() {
        assert_eq!(days_in_year(2000), 366);
        assert_eq!(days_in_year(1900), 365);
    }

    #[test]
    fn private_helpers_are_visible_to_child_modules() {
        assert_eq!(year_length(true), 366);
    }
}
