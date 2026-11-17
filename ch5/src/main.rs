//! Chapter 5: Ownership, borrowing and lifetimes.

#[derive(Debug, Clone, Copy, PartialEq)]
struct Equatorial {
    ra_deg: f64,
    dec_deg: f64,
}

/// Owns its name: the `String` is freed when the `Star` is dropped.
#[derive(Debug, Clone, PartialEq)]
struct Star {
    name: String,
    position: Equatorial,
}

// ---- three ways to take a parameter ----------------------------------------

/// Takes ownership: the caller cannot use the vector afterwards.
fn into_names(stars: Vec<Star>) -> Vec<String> {
    stars.into_iter().map(|s| s.name).collect()
}

/// Shared borrow: read-only access, any number at once.
fn mean_ra(stars: &[Star]) -> f64 {
    stars.iter().map(|s| s.position.ra_deg).sum::<f64>() / stars.len() as f64
}

/// Exclusive borrow: read and write, but only one at a time.
fn shift_ra(stars: &mut [Star], delta_deg: f64) {
    for star in stars.iter_mut() {
        star.position.ra_deg = (star.position.ra_deg + delta_deg).rem_euclid(360.0);
    }
}

// ---- lifetimes -------------------------------------------------------------

/// The returned `&str` borrows from `line`; elision infers that it lives as long as `line`.
fn parse_line(line: &str) -> Option<(&str, f64, f64)> {
    let mut fields = line.split_whitespace();
    let name = fields.next()?;
    let ra_hours = fields.next()?.parse().ok()?;
    let dec = fields.next()?.parse().ok()?;
    Some((name, ra_hours, dec))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Reign {
    name: &'static str,
    regnal_years: u32,
}

const fn r(name: &'static str, regnal_years: u32) -> Reign {
    Reign { name, regnal_years }
}

const REIGNS: [Reign; 8] = [
    r("Nabunasir", 14),
    r("Nabu-nadin-zeri", 2),
    r("Mukin-zeri and Pul", 5),
    r("Ululayu", 5),
    r("Mardukempad", 12),
    r("Sargon II", 5),
    r("First Kingless period", 2),
    r("Bel-ibni", 3),
];

/// `'static` data lives for the whole program, so the result does not depend on `name`.
fn find_reign(name: &str) -> Option<&'static Reign> {
    REIGNS.iter().find(|r| r.name == name)
}

/// Two inputs, so elision cannot guess: the result lives as long as both do.
fn longest_reign<'a>(a: &'a Reign, b: &'a Reign) -> &'a Reign {
    if b.regnal_years > a.regnal_years {
        b
    } else {
        a
    }
}

/// A struct that borrows: it cannot outlive the text it points into.
struct Catalogue<'a> {
    entries: Vec<(&'a str, f64, f64)>,
}

impl<'a> Catalogue<'a> {
    fn parse(text: &'a str) -> Self {
        Self {
            entries: text.lines().filter_map(parse_line).collect(),
        }
    }

    fn names(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.entries.iter().map(|(name, _, _)| *name)
    }
}

fn main() {
    // Move.
    let a = String::from("Vega");
    let b = a; // `a` is moved into `b`
    println!("{b}");
    // println!("{a}"); // error[E0382]: borrow of moved value: `a`

    // Clone: an explicit deep copy.
    let c = b.clone();
    println!("{b} {c}");

    // Copy: small plain-data types are copied implicitly.
    let p = Equatorial {
        ra_deg: 279.23,
        dec_deg: 38.78,
    };
    let q = p;
    println!("{p:?} {q:?}");

    let mut stars = vec![
        Star {
            name: "Vega".into(),
            position: p,
        },
        Star {
            name: "Antares".into(),
            position: Equatorial {
                ra_deg: 247.35,
                dec_deg: -26.43,
            },
        },
    ];
    println!("mean RA = {:.3}", mean_ra(&stars));
    shift_ra(&mut stars, 90.0);
    println!("mean RA after shift = {:.3}", mean_ra(&stars));

    // Borrow rules: many `&` or one `&mut`, never both at once.
    let first = &stars[0];
    println!("first = {}", first.name);
    // `first` is no longer used, so a `&mut` borrow is allowed here.
    shift_ra(&mut stars, 0.0);

    let names = into_names(stars); // `stars` is moved
    println!("{names:?}");
    // println!("{}", stars.len()); // error[E0382]: borrow of moved value: `stars`

    // Lifetimes.
    let text = "Vega 18.615 38.783\nAntares 16.490 -26.432\nbroken line\nSpica 13.420 -11.161";
    let catalogue = Catalogue::parse(text);
    println!(
        "{} usable lines: {:?}",
        catalogue.entries.len(),
        catalogue.names().collect::<Vec<_>>()
    );

    let nabunasir = find_reign("Nabunasir").expect("known reign");
    let mardukempad = find_reign("Mardukempad").expect("known reign");
    println!("longest: {}", longest_reign(nabunasir, mardukempad).name);
    println!("unknown: {:?}", find_reign("Nobody"));

    // A reference may not outlive its owner:
    // let dangling: &String;
    // { let s = String::from("x"); dangling = &s; } // error[E0597]: `s` does not live long enough
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_wraps_into_range() {
        let mut stars = vec![Star {
            name: "X".into(),
            position: Equatorial {
                ra_deg: 350.0,
                dec_deg: 0.0,
            },
        }];
        shift_ra(&mut stars, 20.0);
        assert!((stars[0].position.ra_deg - 10.0).abs() < 1e-12);
    }

    #[test]
    fn catalogue_skips_broken_lines() {
        let text = "Vega 18.6 38.8\nnot a star\nSpica 13.4 -11.2";
        let names: Vec<_> = Catalogue::parse(text).names().collect();
        assert_eq!(names, ["Vega", "Spica"]);
    }

    #[test]
    fn longest_reign_prefers_the_first_on_ties() {
        let a = find_reign("Ululayu").unwrap();
        let b = find_reign("Mukin-zeri and Pul").unwrap();
        assert_eq!(longest_reign(a, b).name, "Ululayu");
    }
}
