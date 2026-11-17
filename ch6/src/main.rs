//! Chapter 6: Traits. `Display`, operators, ordering, `FromStr`, `From` and your own traits.

use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use std::str::FromStr;

const SECS_PER_MINUTE: i64 = 60;
const SECS_PER_HOUR: i64 = 3_600;
const SECS_PER_DAY: i64 = 86_400;
const SECS_PER_YEAR: i64 = 365 * SECS_PER_DAY;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Sign {
    Plus,
    Minus,
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Sign::Plus => "+",
            Sign::Minus => "-",
        })
    }
}

/// Derived `PartialOrd`/`Ord` compare field by field. With one field, that is the signed total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
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
    fn hours(&self) -> u32 {
        (self.magnitude() % SECS_PER_DAY as u64 / SECS_PER_HOUR as u64) as u32
    }
    fn minutes(&self) -> u32 {
        (self.magnitude() % SECS_PER_HOUR as u64 / SECS_PER_MINUTE as u64) as u32
    }
    fn secs(&self) -> u32 {
        (self.magnitude() % SECS_PER_MINUTE as u64) as u32
    }

    fn to_days_f64(self) -> f64 {
        self.total_secs as f64 / SECS_PER_DAY as f64
    }
}

// ---- Display ---------------------------------------------------------------

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} years {:02} days {:02} hours {:02} minutes {:02} seconds",
            self.sign(),
            self.years(),
            self.days(),
            self.hours(),
            self.minutes(),
            self.secs()
        )
    }
}

// ---- Operators -------------------------------------------------------------

impl Neg for Duration {
    type Output = Duration;
    fn neg(self) -> Duration {
        Duration::from_total_secs(-self.total_secs)
    }
}

impl Add for Duration {
    type Output = Duration;
    fn add(self, rhs: Duration) -> Duration {
        Duration::from_total_secs(self.total_secs + rhs.total_secs)
    }
}

impl Sub for Duration {
    type Output = Duration;
    fn sub(self, rhs: Duration) -> Duration {
        Duration::from_total_secs(self.total_secs - rhs.total_secs)
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

// ---- From / Into -----------------------------------------------------------

impl From<i64> for Duration {
    fn from(total_secs: i64) -> Self {
        Duration::from_total_secs(total_secs)
    }
}

// ---- FromStr ---------------------------------------------------------------

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

    fn name(self) -> &'static str {
        match self {
            EgyptianMonth::Thoth => "Thoth",
            EgyptianMonth::Phaophi => "Phaophi",
            EgyptianMonth::Athyr => "Athyr",
            EgyptianMonth::Choiak => "Choiak",
            EgyptianMonth::Tybi => "Tybi",
            EgyptianMonth::Mechir => "Mechir",
            EgyptianMonth::Phamenoth => "Phamenoth",
            EgyptianMonth::Pharmouthi => "Pharmouthi",
            EgyptianMonth::Pachon => "Pachon",
            EgyptianMonth::Payni => "Payni",
            EgyptianMonth::Epiphi => "Epiphi",
            EgyptianMonth::Mesore => "Mesore",
            EgyptianMonth::Epagomenal => "Epagomenal",
        }
    }
}

impl fmt::Display for EgyptianMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseMonthError(String);

impl fmt::Display for ParseMonthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown Egyptian month {:?}", self.0)
    }
}

impl std::error::Error for ParseMonthError {}

impl FromStr for EgyptianMonth {
    type Err = ParseMonthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|m| m.name() == s)
            .ok_or_else(|| ParseMonthError(s.to_owned()))
    }
}

// ---- A trait of our own ----------------------------------------------------

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

/// A point on the celestial sphere in some coordinate system.
trait SkyPosition {
    fn system(&self) -> &'static str;
    fn longitude_like(&self) -> f64;
    fn latitude_like(&self) -> f64;

    /// A default method; implementors may override it.
    fn describe(&self) -> String {
        format!(
            "{}: {:.3} / {:.3}",
            self.system(),
            self.longitude_like(),
            self.latitude_like()
        )
    }
}

impl SkyPosition for Equatorial {
    fn system(&self) -> &'static str {
        "equatorial"
    }
    fn longitude_like(&self) -> f64 {
        self.ra_deg
    }
    fn latitude_like(&self) -> f64 {
        self.dec_deg
    }
}

impl SkyPosition for Ecliptic {
    fn system(&self) -> &'static str {
        "ecliptic"
    }
    fn longitude_like(&self) -> f64 {
        self.lon_deg
    }
    fn latitude_like(&self) -> f64 {
        self.lat_deg
    }
    fn describe(&self) -> String {
        format!(
            "ecliptic (lambda, beta) = ({:.3}, {:.3})",
            self.lon_deg, self.lat_deg
        )
    }
}

/// Static dispatch: the compiler generates one copy per concrete type.
fn print_generic<P: SkyPosition>(p: &P) {
    println!("generic : {}", p.describe());
}

/// Dynamic dispatch: one function, a vtable lookup at run time.
fn print_all(points: &[Box<dyn SkyPosition>]) {
    for p in points {
        println!("dyn     : {}", p.describe());
    }
}

fn d(sign: Sign, years: i64, days: i64, hours: i64, minutes: i64, secs: i64) -> Duration {
    let magnitude =
        years * SECS_PER_YEAR + days * SECS_PER_DAY + hours * SECS_PER_HOUR + minutes * 60 + secs;
    Duration::from_total_secs(match sign {
        Sign::Plus => magnitude,
        Sign::Minus => -magnitude,
    })
}

fn main() {
    let a = d(Sign::Plus, 1, 364, 23, 59, 59);
    let b = d(Sign::Plus, 0, 0, 0, 0, 1);
    println!("a       = {a}");
    println!("a + b   = {}", a + b);
    println!("b - a   = {}", b - a);
    println!("-a      = {}", -a);

    let mut acc = Duration::default();
    acc += a;
    acc -= d(Sign::Plus, 3, 0, 0, 0, 0);
    println!("acc     = {acc}");
    println!("a < b?    {}", a < b);
    println!("max       {}", a.max(b));
    println!("days    = {}", acc.to_days_f64());

    let from_int: Duration = 3_600.into(); // `From<i64>` gives `Into` for free
    println!("1 hour  = {from_int}");

    let mut sorted = [b, a, -a, Duration::default()];
    sorted.sort();
    for x in sorted {
        println!("sorted  : {x}");
    }

    let month: EgyptianMonth = "Payni".parse().unwrap();
    println!("{month} is month {}", month.number());
    println!("{:?}", "Smarch".parse::<EgyptianMonth>());
    if let Err(e) = "Smarch".parse::<EgyptianMonth>() {
        println!("{e}");
    }

    let eq = Equatorial {
        ra_deg: 116.329,
        dec_deg: 28.026,
    };
    let ecl = Ecliptic {
        lon_deg: 113.216,
        lat_deg: 6.684,
    };
    print_generic(&eq);
    print_generic(&ecl);
    print_all(&[Box::new(eq), Box::new(ecl)]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operators_do_not_mutate_operands() {
        let a = d(Sign::Plus, 1, 10, 0, 0, 0);
        let b = d(Sign::Plus, 2, 20, 0, 0, 0);
        assert_eq!(a - b, d(Sign::Minus, 1, 10, 0, 0, 0));
        assert_eq!(a, d(Sign::Plus, 1, 10, 0, 0, 0));
    }

    #[test]
    fn ordering_is_by_signed_total() {
        assert!(d(Sign::Minus, 5, 0, 0, 0, 0) < d(Sign::Minus, 1, 0, 0, 0, 0));
        assert!(d(Sign::Minus, 1, 0, 0, 0, 0) < d(Sign::Plus, 1, 0, 0, 0, 0));
    }

    #[test]
    fn display_format() {
        assert_eq!(
            d(Sign::Plus, 5, 3, 2, 4, 6).to_string(),
            "+ 5 years 03 days 02 hours 04 minutes 06 seconds"
        );
    }

    #[test]
    fn month_parse_and_display_round_trip() {
        for m in EgyptianMonth::ALL {
            assert_eq!(m.to_string().parse::<EgyptianMonth>(), Ok(m));
        }
        assert!("Smarch".parse::<EgyptianMonth>().is_err());
    }

    #[test]
    fn trait_default_and_override() {
        let eq = Equatorial {
            ra_deg: 1.0,
            dec_deg: 2.0,
        };
        let ecl = Ecliptic {
            lon_deg: 1.0,
            lat_deg: 2.0,
        };
        assert!(eq.describe().starts_with("equatorial"));
        assert!(ecl.describe().contains("lambda"));
    }
}
