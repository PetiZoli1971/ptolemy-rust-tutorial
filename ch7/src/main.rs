//! Chapter 7: Generics, iterators and closures.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Reign {
    name: &'static str,
    regnal_years: u32,
}

const fn r(name: &'static str, regnal_years: u32) -> Reign {
    Reign { name, regnal_years }
}

const REIGNS: [Reign; 17] = [
    r("Nabunasir", 14),
    r("Nabu-nadin-zeri", 2),
    r("Mukin-zeri and Pul", 5),
    r("Ululayu", 5),
    r("Mardukempad", 12),
    r("Sargon II", 5),
    r("First Kingless period", 2),
    r("Bel-ibni", 3),
    r("Ashur-nadin-shumi", 6),
    r("Nergal-ushezib", 1),
    r("Mushezib-Marduk", 4),
    r("Second Kingless period", 8),
    r("Esarhaddon", 13),
    r("Shamash-shuma-ukin", 20),
    r("Kandalanu", 22),
    r("Nabonpolassar", 21),
    r("Nebuchadnezzar", 43),
];

// ---- generics --------------------------------------------------------------

/// A value together with a quality indicator. `T` is whatever was converted.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Checked<T> {
    value: T,
    gradient: f64,
}

impl<T> Checked<T> {
    /// Transform the payload and keep the metadata. `U` is chosen by the closure.
    fn map<U>(self, f: impl FnOnce(T) -> U) -> Checked<U> {
        Checked {
            value: f(self.value),
            gradient: self.gradient,
        }
    }
}

/// The trait bounds say what `T` must be able to do.
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter().copied();
    let first = iter.next()?;
    Some(iter.fold(first, |best, x| if x > best { x } else { best }))
}

/// A `where` clause says the same thing and reads better for long bounds.
fn mean<T>(items: &[T]) -> Option<f64>
where
    T: Copy + Into<f64>,
{
    if items.is_empty() {
        return None;
    }
    let sum: f64 = items.iter().map(|&x| x.into()).sum();
    Some(sum / items.len() as f64)
}

// ---- iterators -------------------------------------------------------------

/// Loop version.
fn cumulated_years_loop(name: &str) -> Option<u32> {
    let mut total = 0;
    for reign in REIGNS.iter() {
        total += reign.regnal_years;
        if reign.name == name {
            return Some(total);
        }
    }
    None
}

/// Iterator version: `scan` carries a running total, `find` stops at the first match.
fn cumulated_years(name: &str) -> Option<u32> {
    REIGNS
        .iter()
        .scan(0, |total, reign| {
            *total += reign.regnal_years;
            Some((reign.name, *total))
        })
        .find(|(n, _)| *n == name)
        .map(|(_, total)| total)
}

/// Returning `impl Iterator` hides the concrete (and unnameable) iterator type.
fn reigns_longer_than(years: u32) -> impl Iterator<Item = &'static Reign> {
    REIGNS.iter().filter(move |r| r.regnal_years > years)
}

// ---- closures --------------------------------------------------------------

/// Takes any closure `f: (f64, f64) -> (f64, f64)` and applies it pairwise (as in `ptolemy_astro::batch`).
fn map_pairs(first: &[f64], second: &[f64], f: impl Fn(f64, f64) -> (f64, f64)) -> Vec<(f64, f64)> {
    first.iter().zip(second).map(|(&a, &b)| f(a, b)).collect()
}

/// Returns a closure that captures `factor` by move.
fn scaler(factor: f64) -> impl Fn(f64) -> f64 {
    move |x| x * factor
}

fn main() {
    // Generic struct.
    let c = Checked {
        value: 18.6156_f64,
        gradient: 1.2,
    };
    let hours_to_deg = c.map(|h| h * 15.0);
    let as_text = hours_to_deg.map(|d| format!("{d:.3} deg"));
    println!("{c:?}\n{hours_to_deg:?}\n{as_text:?}");

    println!("largest i32: {:?}", largest(&[3, 9, 4]));
    println!("largest f64: {:?}", largest(&[0.5, -2.0, 7.25]));
    println!("largest of nothing: {:?}", largest::<u8>(&[]));
    println!("mean of u8s: {:?}", mean(&[1_u8, 2, 4]));
    println!("mean of f32s: {:?}", mean(&[0.5_f32, 1.5]));

    // Iterator adaptors.
    println!("\ncumulated years:");
    for name in ["Nabunasir", "Kandalanu", "Nebuchadnezzar", "Nobody"] {
        println!(
            "{name:<16} {:?} {:?}",
            cumulated_years(name),
            cumulated_years_loop(name)
        );
    }

    let total: u32 = REIGNS.iter().map(|r| r.regnal_years).sum();
    println!("\ntotal regnal years: {total}");

    let long: Vec<_> = reigns_longer_than(20).map(|r| r.name).collect();
    println!("longer than 20 years: {long:?}");

    let (kingless, kings): (Vec<&Reign>, Vec<&Reign>) =
        REIGNS.iter().partition(|r| r.name.contains("Kingless"));
    println!(
        "{} kingless periods, {} reigns",
        kingless.len(),
        kings.len()
    );

    let longest = REIGNS.iter().max_by_key(|r| r.regnal_years).unwrap();
    println!("longest: {} ({} years)", longest.name, longest.regnal_years);

    let first_three: Vec<_> = REIGNS.iter().take(3).map(|r| r.name).collect();
    println!("first three: {first_three:?}");
    let backwards: Vec<_> = REIGNS.iter().rev().take(2).map(|r| r.name).collect();
    println!("last two: {backwards:?}");

    // Floats are only `PartialOrd`: use `fold` with `f64::max`, or `total_cmp` to sort.
    let ra = [18.6156, 16.4901, 13.4199, 10.1396];
    let max = ra.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut sorted = ra;
    sorted.sort_by(|a, b| a.total_cmp(b));
    println!("max RA {max}, sorted {sorted:?}");

    // Closures.
    let dec = [38.78, -26.43, -11.16, 11.97];
    let eps = 23.44_f64.to_radians(); // captured by reference below
    let out = map_pairs(&ra, &dec, |r, d| (r * 15.0, d + eps.sin()));
    for (r, d) in &out {
        println!("{r:>8.3} {d:>8.3}");
    }

    let to_degrees = scaler(15.0);
    println!("6 h = {} deg", to_degrees(6.0));

    // A list of different closures needs `Box<dyn Fn>`.
    let pipeline: Vec<Box<dyn Fn(f64) -> f64>> = vec![
        Box::new(|x| x * 15.0),
        Box::new(|x| x + 360.0),
        Box::new(|x| x.rem_euclid(360.0)),
    ];
    let result = pipeline.iter().fold(-6.0, |acc, f| f(acc));
    println!("pipeline(-6 h) = {result}");

    // A closure that mutates captured state is `FnMut`.
    let mut calls = 0;
    let mut counted = |x: f64| {
        calls += 1;
        x.to_radians()
    };
    counted(90.0);
    counted(180.0);
    println!("closure called {calls} times");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_versions_agree_for_every_reign() {
        for reign in REIGNS {
            assert_eq!(
                cumulated_years(reign.name),
                cumulated_years_loop(reign.name)
            );
        }
        assert_eq!(cumulated_years("Kandalanu"), Some(122));
        assert_eq!(cumulated_years("Nobody"), None);
    }

    #[test]
    fn cumulated_years_are_the_running_sum() {
        let mut total = 0;
        for reign in REIGNS {
            total += reign.regnal_years;
            assert_eq!(cumulated_years(reign.name), Some(total));
        }
    }

    #[test]
    fn largest_and_mean() {
        assert_eq!(largest(&[1, 5, 2]), Some(5));
        assert_eq!(largest::<f64>(&[]), None);
        assert_eq!(mean(&[1_u8, 3]), Some(2.0));
    }

    #[test]
    fn checked_map_keeps_the_gradient() {
        let c = Checked {
            value: 2,
            gradient: 0.7,
        }
        .map(|v| v * 2);
        assert_eq!((c.value, c.gradient), (4, 0.7));
    }

    #[test]
    fn map_pairs_applies_the_closure() {
        let out = map_pairs(&[1.0, 2.0], &[10.0, 20.0], |a, b| (a + b, a * b));
        assert_eq!(out, [(11.0, 10.0), (22.0, 40.0)]);
    }
}
