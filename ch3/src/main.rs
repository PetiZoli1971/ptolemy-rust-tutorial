//! Chapter 3: Custom types. `struct`, `enum`, `impl` and `derive`.

// ---- structs ---------------------------------------------------------------

/// Classic struct with named fields. Angles are in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Equatorial {
    ra_deg: f64,
    dec_deg: f64,
}

impl Equatorial {
    /// An associated function (no `self`): called as `Equatorial::new(..)`.
    fn new(ra_deg: f64, dec_deg: f64) -> Self {
        Self { ra_deg, dec_deg }
    }

    fn from_hours(ra_hours: f64, dec_deg: f64) -> Self {
        Self::new(ra_hours * 15.0, dec_deg)
    }

    /// A method: the first parameter is `self`, here borrowed.
    fn ra_hours(&self) -> f64 {
        self.ra_deg / 15.0
    }
}

/// `Default` is derived: every field gets its own default (0.0 for `f64`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Ecliptic {
    lon_deg: f64,
    lat_deg: f64,
}

/// Tuple struct used as a "newtype": a distinct type around an `f64`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Degrees(f64);

/// A unit-like struct carries no data; it is only a type.
#[derive(Debug)]
struct J2000;

// ---- enums -----------------------------------------------------------------

/// A plain enum: one of a fixed set of choices (replaces the string `'HOUR'`/`'DEG'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RaUnit {
    Hours,
    Degrees,
}

impl RaUnit {
    fn to_degrees(self, ra: f64) -> f64 {
        match self {
            RaUnit::Hours => ra * 15.0,
            RaUnit::Degrees => ra,
        }
    }
}

/// An enum whose variants carry data.
#[derive(Debug, Clone, Copy, PartialEq)]
enum RightAscension {
    Hours(f64),
    Degrees(f64),
    HoursMinutesSeconds { h: u8, m: u8, s: f64 },
}

impl RightAscension {
    fn to_degrees(self) -> f64 {
        match self {
            RightAscension::Hours(h) => h * 15.0,
            RightAscension::Degrees(d) => d,
            RightAscension::HoursMinutesSeconds { h, m, s } => {
                (f64::from(h) + f64::from(m) / 60.0 + s / 3600.0) * 15.0
            }
        }
    }
}

/// The 13 Egyptian months; `as u8` gives the discriminant 0, 1, 2, ...
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EgyptianMonth {
    Thoth,
    Phaophi,
    Athyr,
    Choiak,
    Tybi,
    Mechir,
    Phamenoth,
    Pharmouthi,
    Pachon,
    Payni,
    Epiphi,
    Mesore,
    Epagomenal,
}

impl EgyptianMonth {
    /// All variants in calendar order; enums cannot be iterated directly.
    const ALL: [EgyptianMonth; 13] = [
        EgyptianMonth::Thoth,
        EgyptianMonth::Phaophi,
        EgyptianMonth::Athyr,
        EgyptianMonth::Choiak,
        EgyptianMonth::Tybi,
        EgyptianMonth::Mechir,
        EgyptianMonth::Phamenoth,
        EgyptianMonth::Pharmouthi,
        EgyptianMonth::Pachon,
        EgyptianMonth::Payni,
        EgyptianMonth::Epiphi,
        EgyptianMonth::Mesore,
        EgyptianMonth::Epagomenal,
    ];

    fn number(self) -> u8 {
        self as u8 + 1
    }

    fn length(self) -> u8 {
        if self == EgyptianMonth::Epagomenal {
            5
        } else {
            30
        }
    }
}

// ---- encapsulation ---------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    Plus,
    Minus,
}

const SECS_PER_DAY: i64 = 86_400;
const SECS_PER_YEAR: i64 = 365 * SECS_PER_DAY;

/// The field is private (no `pub`), so every value goes through our constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Duration {
    total_secs: i64,
}

impl Duration {
    const fn from_total_secs(total_secs: i64) -> Self {
        Self { total_secs }
    }

    fn sign(&self) -> Sign {
        if self.total_secs < 0 {
            Sign::Minus
        } else {
            Sign::Plus
        }
    }

    fn magnitude(&self) -> u64 {
        self.total_secs.unsigned_abs()
    }

    fn years(&self) -> u64 {
        self.magnitude() / SECS_PER_YEAR as u64
    }

    fn days(&self) -> u32 {
        (self.magnitude() % SECS_PER_YEAR as u64 / SECS_PER_DAY as u64) as u32
    }
}

// ---- a constant table of structs -------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Reign {
    name: &'static str,
    regnal_years: u32,
}

const fn r(name: &'static str, regnal_years: u32) -> Reign {
    Reign { name, regnal_years }
}

const REIGNS: [Reign; 4] = [
    r("Nabunasir", 14),
    r("Nabu-nadin-zeri", 2),
    r("Mukin-zeri and Pul", 5),
    r("Ululayu", 5),
];

fn main() {
    let vega = Equatorial::from_hours(18.615_607_22, 38.782_993_11);
    println!("{vega:?}");
    println!("{vega:#?}"); // pretty debug
    println!("Vega RA = {:.6} h", vega.ra_hours());

    // `Copy` lets us keep using `vega` after passing it by value.
    let copy = vega;
    println!("copies are equal: {}", copy == vega);

    // Struct update syntax: copy the rest from another value.
    let shifted = Equatorial {
        dec_deg: -26.43,
        ..vega
    };
    println!("{shifted:?}");

    let origin = Ecliptic::default();
    println!("{origin:?}");
    let pollux = Ecliptic {
        lon_deg: 113.2,
        ..Default::default()
    };
    println!("{pollux:?}");

    let Degrees(inner) = Degrees(45.0); // destructuring a tuple struct
    println!("{inner} {:?} {J2000:?}", Degrees(90.0));

    for unit in [RaUnit::Hours, RaUnit::Degrees] {
        println!("{unit:?}: 6.0 -> {} deg", unit.to_degrees(6.0));
    }

    let angles = [
        RightAscension::Hours(6.0),
        RightAscension::Degrees(90.0),
        RightAscension::HoursMinutesSeconds { h: 6, m: 0, s: 0.0 },
    ];
    for a in angles {
        println!("{a:?} = {} deg", a.to_degrees());
    }

    let m = EgyptianMonth::Epagomenal;
    println!("{m:?}: number {}, length {}", m.number(), m.length());
    println!("Payni is month {}", EgyptianMonth::Payni.number());
    let days: u32 = EgyptianMonth::ALL
        .iter()
        .map(|m| u32::from(m.length()))
        .sum();
    println!("{} months, {days} days in a year", EgyptianMonth::ALL.len());

    let d = Duration::from_total_secs(-(2 * SECS_PER_YEAR + 3 * SECS_PER_DAY));
    println!(
        "{d:?}: {:?} {} years {} days",
        d.sign(),
        d.years(),
        d.days()
    );

    for reign in REIGNS {
        println!("{:<20} {:>2} years", reign.name, reign.regnal_years);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hours_are_fifteen_degrees() {
        assert_eq!(
            Equatorial::from_hours(6.0, 10.0),
            Equatorial::new(90.0, 10.0)
        );
    }

    #[test]
    fn month_numbers_follow_declaration_order() {
        assert_eq!(EgyptianMonth::Thoth.number(), 1);
        assert_eq!(EgyptianMonth::Epagomenal.number(), 13);
    }

    #[test]
    fn negative_duration_splits_into_components() {
        let d = Duration::from_total_secs(-(SECS_PER_YEAR + SECS_PER_DAY));
        assert_eq!(d.sign(), Sign::Minus);
        assert_eq!((d.years(), d.days()), (1, 1));
    }

    #[test]
    fn right_ascension_variants_agree() {
        let a = RightAscension::Hours(6.5).to_degrees();
        let b = RightAscension::HoursMinutesSeconds {
            h: 6,
            m: 30,
            s: 0.0,
        }
        .to_degrees();
        assert!((a - b).abs() < 1e-12);
    }
}
