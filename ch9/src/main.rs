//! Chapter 9: Numerics. Angles, solar/lunar parameters, coordinate conversion and precession.

const J2000: f64 = 2_451_545.0;
const DAYS_PER_CENTURY: f64 = 36_525.0;

// ---- angles ----------------------------------------------------------------

/// Maps any angle into `[0, 360)`.
fn normalize_deg(angle: f64) -> f64 {
    let wrapped = angle.rem_euclid(360.0);
    // rem_euclid can return exactly 360.0 for tiny negative inputs.
    if wrapped >= 360.0 {
        0.0
    } else {
        wrapped
    }
}

/// Signed shortest difference `a - b` in degrees, in `[-180, 180)`.
fn angle_diff(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
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

// ---- Julian Day ------------------------------------------------------------

/// Float Julian Day (Meeus ch. 7); `gregorian` selects the calendar of the input date.
fn julian_day(year: i32, month: i32, day: f64, gregorian: bool) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let b = if gregorian {
        let a = (f64::from(y) / 100.0).floor();
        2.0 - a + (a / 4.0).floor()
    } else {
        0.0
    };
    (365.25 * f64::from(y + 4716)).floor() + (30.6001 * f64::from(m + 1)).floor() + day + b - 1524.5
}

// ---- solar and lunar parameters (Meeus ch. 22, 25) --------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
struct AstroParams {
    jd: f64,
    t: f64,
}

impl AstroParams {
    fn from_jd(jd: f64) -> Self {
        Self {
            jd,
            t: (jd - J2000) / DAYS_PER_CENTURY,
        }
    }

    fn sun_mean_anomaly(&self) -> f64 {
        let t = self.t;
        357.529_10 + 35_999.050_30 * t - 0.000_155_9 * t * t + 0.000_000_48 * t * t * t
    }

    fn equation_of_center(&self) -> f64 {
        let t = self.t;
        let m = self.sun_mean_anomaly().to_radians();
        (1.914_600 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
            + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
            + 0.000_290 * (3.0 * m).sin()
    }

    /// Mean obliquity of the ecliptic (Meeus 22.2). The cubic term must be divided by 3600.
    fn mean_obliquity(&self) -> f64 {
        let t = self.t;
        23.439_291_111 - 46.815 * t / 3600.0 - 0.000_59 * t * t / 3600.0
            + 0.001_813 * t * t * t / 3600.0
    }

    fn moon_ascending_node_longitude(&self) -> f64 {
        let t = self.t;
        125.044_52 - 1_934.136_261 * t + 0.002_070_8 * t * t + t * t * t / 450_000.0
    }

    fn sun_mean_longitude(&self) -> f64 {
        280.466_5 + 36_000.769_8 * self.t
    }

    fn moon_mean_longitude(&self) -> f64 {
        218.316_5 + 481_267.881_3 * self.t
    }

    fn nutation_longitude(&self) -> f64 {
        let omega = self.moon_ascending_node_longitude();
        let sun = self.sun_mean_longitude();
        let moon = self.moon_mean_longitude();
        let arcsec = -17.2 * omega.to_radians().sin()
            - 1.32 * (2.0 * sun).to_radians().sin()
            - 0.23 * (2.0 * moon).to_radians().sin()
            + 0.21 * (2.0 * omega).to_radians().sin();
        arcsec / 3600.0
    }

    fn nutation_obliquity(&self) -> f64 {
        let omega = self.moon_ascending_node_longitude();
        let sun = self.sun_mean_longitude();
        let moon = self.moon_mean_longitude();
        let arcsec = 9.2 * omega.to_radians().cos()
            + 0.57 * (2.0 * sun).to_radians().cos()
            + 0.1 * (2.0 * moon).to_radians().cos()
            - 0.09 * (2.0 * omega).to_radians().cos();
        arcsec / 3600.0
    }

    /// Mean obliquity plus nutation in obliquity, in degrees.
    fn obliquity(&self) -> f64 {
        self.mean_obliquity() + self.nutation_obliquity()
    }
}

// ---- coordinate conversion -------------------------------------------------

fn equatorial_to_ecliptic_at(eq: Equatorial, obliquity_deg: f64) -> Ecliptic {
    let (sin_a, cos_a) = eq.ra_deg.to_radians().sin_cos();
    let (sin_d, cos_d) = eq.dec_deg.to_radians().sin_cos();
    let (sin_e, cos_e) = obliquity_deg.to_radians().sin_cos();
    let y = sin_a * cos_e * cos_d + sin_d * sin_e;
    let x = cos_a * cos_d;
    // `asin` is only defined on [-1, 1]; rounding can push the value just outside.
    let sin_beta = (sin_d * cos_e - cos_d * sin_e * sin_a).clamp(-1.0, 1.0);
    Ecliptic {
        lon_deg: normalize_deg(y.atan2(x).to_degrees()),
        lat_deg: sin_beta.asin().to_degrees(),
    }
}

fn ecliptic_to_equatorial_at(ecl: Ecliptic, obliquity_deg: f64) -> Equatorial {
    let (sin_l, cos_l) = ecl.lon_deg.to_radians().sin_cos();
    let (sin_b, cos_b) = ecl.lat_deg.to_radians().sin_cos();
    let (sin_e, cos_e) = obliquity_deg.to_radians().sin_cos();
    let y = sin_l * cos_e * cos_b - sin_b * sin_e;
    let x = cos_l * cos_b;
    let sin_delta = (sin_b * cos_e + cos_b * sin_e * sin_l).clamp(-1.0, 1.0);
    Equatorial {
        ra_deg: normalize_deg(y.atan2(x).to_degrees()),
        dec_deg: sin_delta.asin().to_degrees(),
    }
}

// ---- precession ------------------------------------------------------------

/// Rotation angles (degrees) taking coordinates of one epoch to another (Meeus 21.2).
#[derive(Debug, Clone, Copy, PartialEq)]
struct EquatorialPrecession {
    zeta: f64,
    z: f64,
    theta: f64,
}

impl EquatorialPrecession {
    /// `big_t`: centuries from J2000 to the data epoch; `t`: centuries from data epoch to target.
    fn new(big_t: f64, t: f64) -> Self {
        let base = 2306.2181 + 1.39656 * big_t - 0.000139 * big_t * big_t;
        Self {
            zeta: (base * t + (0.30188 - 0.000344 * big_t) * t * t + 0.017998 * t * t * t) / 3600.0,
            z: (base * t + (1.09468 + 0.000066 * big_t) * t * t + 0.018203 * t * t * t) / 3600.0,
            theta: ((2004.3109 - 0.85330 * big_t - 0.000217 * big_t * big_t) * t
                - (0.42665 + 0.000217 * big_t) * t * t
                - 0.041833 * t * t * t)
                / 3600.0,
        }
    }

    fn between(data_jd: f64, target_jd: f64) -> Self {
        Self::new(
            (data_jd - J2000) / DAYS_PER_CENTURY,
            (target_jd - data_jd) / DAYS_PER_CENTURY,
        )
    }

    fn apply(&self, eq: Equatorial) -> Equatorial {
        let (sin_t, cos_t) = self.theta.to_radians().sin_cos();
        let (sin_d, cos_d) = eq.dec_deg.to_radians().sin_cos();
        let (sin_az, cos_az) = (eq.ra_deg + self.zeta).to_radians().sin_cos();
        let a = cos_d * sin_az;
        let b = cos_t * cos_d * cos_az - sin_t * sin_d;
        let c = (sin_t * cos_d * cos_az + cos_t * sin_d).clamp(-1.0, 1.0);
        Equatorial {
            ra_deg: normalize_deg(a.atan2(b).to_degrees() + self.z),
            dec_deg: c.asin().to_degrees(),
        }
    }
}

fn main() {
    // Floating-point reality check.
    println!("0.1 + 0.2 == 0.3 ? {}", 0.1 + 0.2 == 0.3);
    println!("|0.1 + 0.2 - 0.3| = {:e}", (0.1_f64 + 0.2 - 0.3).abs());
    println!("-7 % 360       = {}", -7.0_f64 % 360.0);
    println!("-7 rem_euclid  = {}", (-7.0_f64).rem_euclid(360.0));
    println!("tiny negative  = {}", (-1e-20_f64).rem_euclid(360.0));
    println!("normalize_deg  = {}", normalize_deg(-1e-20));
    println!("angle_diff(359, 1) = {}", angle_diff(359.0, 1.0));
    println!("sqrt(-1) is NaN: {}", (-1.0_f64).sqrt().is_nan());

    // Parameters for Meeus example 22.a: 1987-04-10 00:00.
    let params = AstroParams::from_jd(julian_day(1987, 4, 10.0, true));
    println!("\nJD {} T = {:.12}", params.jd, params.t);
    println!("mean obliquity   {:.6} deg", params.mean_obliquity());
    println!(
        "nutation (lon)   {:.3} arcsec",
        params.nutation_longitude() * 3600.0
    );
    println!(
        "nutation (obl)   {:.3} arcsec",
        params.nutation_obliquity() * 3600.0
    );
    println!("equation of centre {:.4} deg", params.equation_of_center());

    // Meeus example 13.a: Pollux.
    let pollux = Equatorial {
        ra_deg: 116.328_942,
        dec_deg: 28.026_183,
    };
    let ecl = equatorial_to_ecliptic_at(pollux, 23.439_291_1);
    let back = ecliptic_to_equatorial_at(ecl, 23.439_291_1);
    println!("\nPollux {pollux:?}\n  -> {ecl:?}\n  -> {back:?}");

    // Almagest-style computation: J2000 catalogue positions at Ptolemy's epoch.
    let j2000 = julian_day(2000, 1, 1.5, true);
    let ptolemy = julian_day(134, 1, 1.5, false); // Julian calendar date
    println!("\nJD(134-01-01 12:00 Julian) = {ptolemy}");
    let obliquity = AstroParams::from_jd(ptolemy).obliquity();
    let rotation = EquatorialPrecession::between(j2000, ptolemy);
    println!("{rotation:?}");

    let stars = [
        ("Vega", 18.615_607_22, 38.782_993_11),
        ("Antares", 16.490_129_86, -26.431_946_08),
        ("Spica", 13.419_890_15, -11.161_244_91),
    ];
    println!("Epoch 134: ecliptic longitude / latitude (degrees)");
    for (name, ra_hours, dec) in stars {
        let eq = rotation.apply(Equatorial {
            ra_deg: ra_hours * 15.0,
            dec_deg: dec,
        });
        let e = equatorial_to_ecliptic_at(eq, obliquity);
        println!("{name:<8} {:>9.4} {:>9.4}", e.lon_deg, e.lat_deg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "{a} != {b} (eps {eps})");
    }

    #[test]
    fn normalize_handles_edge_cases() {
        close(normalize_deg(-7.0), 353.0, 1e-12);
        close(normalize_deg(720.5), 0.5, 1e-12);
        assert_eq!(normalize_deg(-1e-20), 0.0);
    }

    #[test]
    fn angle_diff_takes_the_short_way() {
        close(angle_diff(359.0, 1.0), -2.0, 1e-12);
        close(angle_diff(1.0, 359.0), 2.0, 1e-12);
    }

    #[test]
    fn meeus_example_22a() {
        let p = AstroParams::from_jd(julian_day(1987, 4, 10.0, true));
        close(p.t, -0.127_296_372_348, 1e-12);
        close(
            p.mean_obliquity(),
            23.0 + 26.0 / 60.0 + 27.407 / 3600.0,
            1e-6,
        );
        // The low-accuracy series is good to about 0.1 arcsecond.
        close(p.nutation_longitude() * 3600.0, -3.788, 0.1);
        close(p.nutation_obliquity() * 3600.0, 9.443, 0.1);
    }

    #[test]
    fn meeus_example_13a() {
        let ecl = equatorial_to_ecliptic_at(
            Equatorial {
                ra_deg: 116.328_942,
                dec_deg: 28.026_183,
            },
            23.439_291_1,
        );
        close(ecl.lon_deg, 113.215_630, 1e-5);
        close(ecl.lat_deg, 6.684_170, 1e-5);
    }

    #[test]
    fn conversion_round_trips() {
        for ra in (0..360).step_by(15) {
            for dec in (-80..=80).step_by(20) {
                let eq = Equatorial {
                    ra_deg: f64::from(ra),
                    dec_deg: f64::from(dec),
                };
                let back = ecliptic_to_equatorial_at(equatorial_to_ecliptic_at(eq, 23.44), 23.44);
                close(angle_diff(back.ra_deg, eq.ra_deg), 0.0, 1e-9);
                close(back.dec_deg, eq.dec_deg, 1e-9);
            }
        }
    }

    #[test]
    fn zero_interval_precession_is_the_identity() {
        let rot = EquatorialPrecession::between(J2000, J2000);
        let eq = Equatorial {
            ra_deg: 123.4,
            dec_deg: -45.6,
        };
        let got = rot.apply(eq);
        close(got.ra_deg, eq.ra_deg, 1e-9);
        close(got.dec_deg, eq.dec_deg, 1e-9);
    }
}
