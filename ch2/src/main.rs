//! Chapter 2: Calendar rules. Functions, expressions, control flow, `Option`.

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const MONTH_LENGTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Julian rule up to and including 1582, Gregorian rule afterwards.
fn is_leap_year(year: i32) -> bool {
    if year <= 1582 {
        year % 4 == 0
    } else {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }
}

/// `match` version: every arm is an expression of the same type.
fn days_in_month_match(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if is_leap_year(year) { 29 } else { 28 }),
        _ => None,
    }
}

/// Table version, as used in the real `ptolemy-time` crate.
fn days_in_month(year: i32, month: u8) -> Option<u8> {
    if !(1..=12).contains(&month) {
        return None;
    }
    if month == 2 && is_leap_year(year) {
        return Some(29);
    }
    Some(MONTH_LENGTHS[usize::from(month) - 1])
}

fn month_name(month: u8) -> Option<&'static str> {
    MONTH_NAMES.get(usize::from(month).checked_sub(1)?).copied()
}

fn month_number(name: &str) -> Option<u8> {
    MONTH_NAMES
        .iter()
        .position(|n| *n == name)
        .map(|i| (i + 1) as u8)
}

/// Dates removed by the Gregorian reform: 1582-10-05 to 1582-10-14.
fn is_skipped_by_reform(year: i32, month: u8, day: u8) -> bool {
    matches!((year, month, day), (1582, 10, 5..=14))
}

/// 1-based day number within the year, or `None` for an invalid date.
fn day_of_year(year: i32, month: u8, day: u8) -> Option<u16> {
    let max_day = days_in_month(year, month)?;
    if day == 0 || day > max_day {
        return None;
    }
    let mut total = u16::from(day);
    let mut m = 1;
    while m < month {
        total += u16::from(days_in_month(year, m)?);
        m += 1;
    }
    Some(total)
}

fn first_leap_year_after(start: i32) -> i32 {
    let mut year = start;
    loop {
        year += 1;
        if is_leap_year(year) {
            break year; // `loop` is an expression: `break` carries the value
        }
    }
}

fn main() {
    for year in [-748, 1, 1500, 1582, 1700, 1900, 2000, 2024] {
        let kind = if is_leap_year(year) { "leap" } else { "common" };
        println!("{year:>5}: {kind}");
    }

    println!("\nLeap years 1896..=1912:");
    for year in 1896..=1912 {
        if is_leap_year(year) {
            print!("{year} ");
        }
    }
    println!();

    println!("\nMonth lengths in 2023 and 2024:");
    for month in 1..=12u8 {
        let name = month_name(month).unwrap_or("?");
        println!(
            "{month:>2} {name:<10} {:>2} {:>2}",
            days_in_month(2023, month).unwrap(),
            days_in_month(2024, month).unwrap()
        );
    }

    println!(
        "\nSame answer from both versions: {}",
        days_in_month(2024, 2) == days_in_month_match(2024, 2)
    );

    println!("\nLookups return Option:");
    println!("days_in_month(2024, 13) = {:?}", days_in_month(2024, 13));
    println!("month_name(0)           = {:?}", month_name(0));
    println!("month_number(\"March\")   = {:?}", month_number("March"));
    if let Some(n) = month_number("Smarch") {
        println!("unexpected {n}");
    } else {
        println!("\"Smarch\" is not a month");
    }

    println!("\nReform gap:");
    println!("1582-10-04: {}", is_skipped_by_reform(1582, 10, 4));
    println!("1582-10-10: {}", is_skipped_by_reform(1582, 10, 10));
    println!("1582-10-15: {}", is_skipped_by_reform(1582, 10, 15));

    println!("\nDay of year:");
    println!("2024-03-01: {:?}", day_of_year(2024, 3, 1));
    println!("2023-03-01: {:?}", day_of_year(2023, 3, 1));
    println!("2023-02-29: {:?}", day_of_year(2023, 2, 29));

    println!(
        "\nFirst leap year after 1899: {}",
        first_leap_year_after(1899)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_year_rules() {
        assert!(is_leap_year(-4));
        assert!(is_leap_year(1500)); // Julian
        assert!(!is_leap_year(1700)); // Gregorian
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
    }

    #[test]
    fn match_and_table_versions_agree() {
        for year in [1900, 2000, 2023, 2024] {
            for month in 0..=13 {
                assert_eq!(days_in_month(year, month), days_in_month_match(year, month));
            }
        }
    }

    #[test]
    fn names_round_trip() {
        for m in 1..=12 {
            assert_eq!(month_number(month_name(m).unwrap()), Some(m));
        }
    }

    #[test]
    fn day_of_year_handles_leap_years() {
        assert_eq!(day_of_year(2024, 12, 31), Some(366));
        assert_eq!(day_of_year(2023, 12, 31), Some(365));
        assert_eq!(day_of_year(2023, 2, 29), None);
    }
}
