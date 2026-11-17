//! Chapter 11: Using external crates. `ndarray` for batches, `log` for diagnostics, `thiserror` for errors.

use log::{Level, LevelFilter, Log, Metadata, Record};
use ndarray::{array, s, Array1, ArrayView1, Zip};
use thiserror::Error;

// ---- logging: a facade (`log`) plus a backend chosen by the application -----

/// `log` only defines the macros; the binary decides where the messages go.
struct StdoutLogger;

impl Log for StdoutLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{:<5}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: StdoutLogger = StdoutLogger;

// ---- scalar core -----------------------------------------------------------

fn normalize_deg(angle: f64) -> f64 {
    let wrapped = angle.rem_euclid(360.0);
    if wrapped >= 360.0 {
        0.0
    } else {
        wrapped
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Equatorial {
    ra_deg: f64,
    dec_deg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Ecliptic {
    lon_deg: f64,
    lat_deg: f64,
}

fn equatorial_to_ecliptic_at(eq: Equatorial, obliquity_deg: f64) -> Ecliptic {
    let (sin_a, cos_a) = eq.ra_deg.to_radians().sin_cos();
    let (sin_d, cos_d) = eq.dec_deg.to_radians().sin_cos();
    let (sin_e, cos_e) = obliquity_deg.to_radians().sin_cos();
    let y = sin_a * cos_e * cos_d + sin_d * sin_e;
    let x = cos_a * cos_d;
    let sin_beta = (sin_d * cos_e - cos_d * sin_e * sin_a).clamp(-1.0, 1.0);
    Ecliptic {
        lon_deg: normalize_deg(y.atan2(x).to_degrees()),
        lat_deg: sin_beta.asin().to_degrees(),
    }
}

/// Kept as in the original Python: `(a - b) / (a^2 + b^2)` of the `atan2` arguments.
fn gradient(a: f64, b: f64) -> f64 {
    let n = a * a + b * b;
    a / n - b / n
}

fn equatorial_to_ecliptic_gradient(eq: Equatorial, obliquity_deg: f64) -> f64 {
    let (sin_a, cos_a) = eq.ra_deg.to_radians().sin_cos();
    let (_, cos_e) = obliquity_deg.to_radians().sin_cos();
    let sin_e = obliquity_deg.to_radians().sin();
    let a = sin_a * cos_e + eq.dec_deg.to_radians().tan() * sin_e;
    gradient(a, cos_a)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConversionQuality {
    Normal,
    ErrorAmplified,
    ErrorDamped,
}

impl ConversionQuality {
    fn from_gradient(gradient: f64) -> Self {
        if gradient.abs() > 1.5 {
            ConversionQuality::ErrorAmplified
        } else if gradient.abs() < 0.5 {
            ConversionQuality::ErrorDamped
        } else {
            ConversionQuality::Normal
        }
    }
}

/// Converts, and logs a warning for conversions that distort errors.
fn equatorial_to_ecliptic_checked(
    name: &str,
    eq: Equatorial,
    obliquity_deg: f64,
) -> (Ecliptic, ConversionQuality) {
    let g = equatorial_to_ecliptic_gradient(eq, obliquity_deg);
    let quality = ConversionQuality::from_gradient(g);
    match quality {
        ConversionQuality::Normal => log::debug!("{name}: gradient {g:.3}"),
        ConversionQuality::ErrorAmplified => {
            log::warn!("{name}: errors are amplified (gradient {g:.3})")
        }
        ConversionQuality::ErrorDamped => log::warn!("{name}: errors are damped (gradient {g:.3})"),
    }
    (equatorial_to_ecliptic_at(eq, obliquity_deg), quality)
}

// ---- batch layer -----------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Error)]
enum BatchError {
    #[error("input arrays have different lengths: {first} and {second}")]
    LengthMismatch { first: usize, second: usize },
}

type Pair = (Array1<f64>, Array1<f64>);

/// Applies `f` to two arrays element by element, after checking that they match in length.
fn map_pairs(
    first: ArrayView1<f64>,
    second: ArrayView1<f64>,
    f: impl Fn(f64, f64) -> (f64, f64),
) -> Result<Pair, BatchError> {
    if first.len() != second.len() {
        return Err(BatchError::LengthMismatch {
            first: first.len(),
            second: second.len(),
        });
    }
    let mut out_first = Array1::zeros(first.len());
    let mut out_second = Array1::zeros(first.len());
    Zip::from(&mut out_first)
        .and(&mut out_second)
        .and(first)
        .and(second)
        .for_each(|o1, o2, &a, &b| (*o1, *o2) = f(a, b));
    Ok((out_first, out_second))
}

/// Returns `(longitudes, latitudes)` in degrees.
fn equatorial_to_ecliptic(
    ra_deg: ArrayView1<f64>,
    dec_deg: ArrayView1<f64>,
    obliquity_deg: f64,
) -> Result<Pair, BatchError> {
    map_pairs(ra_deg, dec_deg, |ra, dec| {
        let e = equatorial_to_ecliptic_at(
            Equatorial {
                ra_deg: ra,
                dec_deg: dec,
            },
            obliquity_deg,
        );
        (e.lon_deg, e.lat_deg)
    })
}

fn main() {
    log::set_logger(&LOGGER).expect("logger is set once");
    log::set_max_level(LevelFilter::Info);

    // ndarray basics.
    let ra_hours = array![18.615_607_22, 16.490_129_86, 13.419_890_15];
    let ra_deg = &ra_hours * 15.0; // elementwise arithmetic
    let dec = array![38.782_993_11, -26.431_946_08, -11.161_244_91];
    println!("ra_deg = {ra_deg}");
    println!(
        "sum = {:.3}, mean = {:.3}",
        ra_deg.sum(),
        ra_deg.mean().unwrap()
    );
    println!("first two = {}", ra_deg.slice(s![..2]));
    println!("sines = {:.4}", dec.mapv(|d: f64| d.to_radians().sin()));
    println!("linspace = {}", Array1::linspace(0.0, 360.0, 5));

    // Batch conversion.
    let obliquity = 23.44;
    let (lon, lat) = equatorial_to_ecliptic(ra_deg.view(), dec.view(), obliquity).unwrap();
    println!("lon = {lon:.4}\nlat = {lat:.4}");

    // Errors from mismatched lengths.
    let short = array![1.0, 2.0];
    match equatorial_to_ecliptic(ra_deg.view(), short.view(), obliquity) {
        Ok(_) => unreachable!(),
        Err(e) => println!("error: {e}"),
    }

    // Logging.
    log::info!("converting {} stars", ra_deg.len());
    log::debug!("this line is filtered out: the level is Info");
    for (name, ra, d) in [
        ("Vega", 279.234, 38.783),
        ("equinox", 0.0, 0.0),
        ("far north", 45.0, 70.0),
    ] {
        let (ecl, quality) = equatorial_to_ecliptic_checked(
            name,
            Equatorial {
                ra_deg: ra,
                dec_deg: d,
            },
            obliquity,
        );
        println!(
            "{name:<10} {:>8.3} {:>8.3} {quality:?}",
            ecl.lon_deg, ecl.lat_deg
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_equals_scalar_elementwise() {
        let ra = Array1::linspace(0.0, 359.0, 50);
        let dec = Array1::linspace(-80.0, 80.0, 50);
        let (lon, lat) = equatorial_to_ecliptic(ra.view(), dec.view(), 23.44).unwrap();
        for i in 0..ra.len() {
            let e = equatorial_to_ecliptic_at(
                Equatorial {
                    ra_deg: ra[i],
                    dec_deg: dec[i],
                },
                23.44,
            );
            assert_eq!(lon[i], e.lon_deg);
            assert_eq!(lat[i], e.lat_deg);
        }
    }

    #[test]
    fn empty_arrays_give_empty_results() {
        let empty = Array1::<f64>::zeros(0);
        let (lon, lat) = equatorial_to_ecliptic(empty.view(), empty.view(), 23.44).unwrap();
        assert!(lon.is_empty() && lat.is_empty());
    }

    #[test]
    fn mismatched_lengths_are_an_error() {
        let a = array![1.0, 2.0, 3.0];
        let b = array![1.0];
        assert_eq!(
            equatorial_to_ecliptic(a.view(), b.view(), 23.44),
            Err(BatchError::LengthMismatch {
                first: 3,
                second: 1
            })
        );
    }

    #[test]
    fn quality_thresholds() {
        assert_eq!(
            ConversionQuality::from_gradient(-1.0),
            ConversionQuality::Normal
        );
        assert_eq!(
            ConversionQuality::from_gradient(1.6),
            ConversionQuality::ErrorAmplified
        );
        assert_eq!(
            ConversionQuality::from_gradient(0.49),
            ConversionQuality::ErrorDamped
        );
    }
}
