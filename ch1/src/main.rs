//! Chapter 1: Hello, Ptolemy. A first Julian Day calculator and a tour of the primitives.

const J2000: f64 = 2_451_545.0;
const DAYS_PER_CENTURY: f64 = 36_525.0;

/// Julian Day of a Gregorian calendar date; `day` may carry a fraction (Meeus, ch. 7).
fn julian_day(year: i32, month: i32, day: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = (f64::from(y) / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * f64::from(y + 4716)).floor() + (30.6001 * f64::from(m + 1)).floor() + day + b - 1524.5
}

fn main() {
    println!("Hello, Ptolemy!");

    // Bindings are immutable by default.
    let jd = julian_day(2000, 1, 1.5);
    println!("J2000.0 is Julian Day {jd}");
    assert!((jd - J2000).abs() < 1e-9);

    // Meeus example 7.a: Sputnik 1 was launched at 1957-10-04.81.
    let sputnik = julian_day(1957, 10, 4.81);
    println!("Sputnik 1: JD {sputnik:.2}");

    // Shadowing: the same name, a new binding (here with a new meaning).
    let t = sputnik - J2000;
    let t = t / DAYS_PER_CENTURY;
    println!("T = {t:.9} Julian centuries since J2000.0");

    // Floating point and integers.
    let ra_hours: f64 = 18.615_607_22; // Vega
    let ra_degrees = ra_hours * 15.0;
    let whole_hours = ra_hours as u8; // `as` truncates towards zero
    let minutes = ((ra_hours - f64::from(whole_hours)) * 60.0) as u8;
    println!("Vega: RA {ra_hours} h = {ra_degrees:.4} deg = {whole_hours}h {minutes:02}m");

    // Integer widths matter: 6745 years of seconds does not fit in an i32.
    let secs_per_year: i32 = 365 * 86_400;
    println!(
        "i32::MAX = {}, seconds per year = {secs_per_year}",
        i32::MAX
    );
    println!(
        "6745 years in i32: {:?}",
        secs_per_year.checked_mul(6745) // None means overflow
    );
    let total_secs = i64::from(secs_per_year) * 6745;
    println!("6745 years in i64: {total_secs}");

    // Tuples, arrays and a mutable counter.
    let epoch: (i32, u8, u8) = (134, 1, 1);
    println!("Ptolemy's epoch: {}-{:02}-{:02}", epoch.0, epoch.1, epoch.2);

    let names = ["Vega", "Antares", "Spica"];
    let ra = [18.615_607_22, 16.490_129_86, 13.419_890_15];
    let mut counted = 0;
    for (name, h) in names.iter().zip(ra) {
        println!("{name:<8} {:>9.4} deg", h * 15.0);
        counted += 1;
    }
    println!("{counted} stars");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_is_the_reference_epoch() {
        assert!((julian_day(2000, 1, 1.5) - J2000).abs() < 1e-9);
    }

    #[test]
    fn sputnik_matches_meeus() {
        assert!((julian_day(1957, 10, 4.81) - 2_436_116.31).abs() < 1e-6);
    }
}
