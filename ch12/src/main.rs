//! Chapter 12: Putting it together with the real `ptolemy-time` and `ptolemy-astro` crates.
//!
//! Usage: `cargo run -p ch12 [YEAR]` (default 134, the epoch used by Ptolemy).

use std::error::Error;

use ptolemy_astro::{Equatorial, StarCoord};
use ptolemy_time::egyptian::{Egyptian, EgyptianMonth};
use ptolemy_time::{HistoricDateTime, JdError};

const STARS: [(&str, f64, f64); 3] = [
    ("Vega", 18.615_607_22, 38.782_993_11),
    ("Antares", 16.490_129_86, -26.431_946_08),
    ("Spica", 13.419_890_15, -11.161_244_91),
];

type StarPosition = (&'static str, f64, f64);

/// Ecliptic `(name, longitude, latitude)` in degrees of each star at January 1st, 12:00 of `year`.
fn star_positions(year: i32) -> Result<Vec<StarPosition>, Box<dyn Error>> {
    let j2000 = HistoricDateTime::new(2000, 1, 1, 12, 0, 0)?;
    let epoch = HistoricDateTime::new(year, 1, 1, 12, 0, 0)?;
    let sc = StarCoord::new(epoch)?;

    STARS
        .iter()
        .map(|&(name, ra_hours, dec)| {
            let precessed = sc.precession_high(&j2000, Equatorial::from_hours(ra_hours, dec))?;
            let ecl = sc.equatorial_to_ecliptic(precessed);
            Ok((name, ecl.lon_deg, ecl.lat_deg))
        })
        .collect::<Result<_, JdError>>()
        .map_err(Into::into)
}

fn main() -> Result<(), Box<dyn Error>> {
    let year = match std::env::args().nth(1) {
        Some(arg) => arg.parse()?,
        None => 134,
    };

    let epoch = HistoricDateTime::new(year, 1, 1, 12, 0, 0)?;
    println!("Epoch {epoch} (JD {})", epoch.jd()?);
    println!("{:<8} {:>9} {:>9}", "star", "lambda", "beta");
    for (name, lon, lat) in star_positions(year)? {
        println!("{name:<8} {lon:>9.4} {lat:>9.4}");
    }

    // Two eclipse observations recorded in Hadrian's reign (Almagest IV.9).
    let payni: EgyptianMonth = "Payni".parse()?;
    let choiak: EgyptianMonth = "Choiak".parse()?;
    let first = Egyptian::new("Hadrianus", 17, payni, 20, 23, 15)?;
    let second = Egyptian::new("Hadrianus", 19, choiak, 2, 23, 0)?;
    println!("\nBetween the eclipses: {}", first.delta_duration(&second));
    println!("Backwards:            {}", second.delta_duration(&first));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_match_the_crate_example() {
        let got = star_positions(134).unwrap();
        let (name, lon, lat) = got[0];
        assert_eq!(name, "Vega");
        assert!((lon - 259.4836).abs() < 1e-4, "{lon}");
        assert!((lat - 61.9675).abs() < 1e-4, "{lat}");
    }

    #[test]
    fn epochs_before_julian_day_zero_are_errors() {
        assert!(star_positions(-5000).is_err());
    }
}
