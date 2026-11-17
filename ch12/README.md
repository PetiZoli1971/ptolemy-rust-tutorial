# Chapter 12: Putting it together

Time to use the real thing. This chapter writes a small command-line program against the public
API of the two real crates, `ptolemy-time` and `ptolemy-astro`, instead of the simplified copies of
the earlier chapters. Everything from the tutorial appears: error enums and `?`, `Option`, ownership,
traits, iterators and closures, modules, floating-point care and tests.

**You will learn**

- path dependencies on another workspace
- reading a crate's public API from its documentation
- combining errors of several crates with `Box<dyn Error>`
- collecting an iterator of `Result`s
- command-line arguments with `std::env::args`
- the workspace tooling: `fmt`, `clippy`, `doc`, `test`
- where to go next

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch12` or `cargo run -p ch12 -- 2000`.

**Requirement:** this chapter needs the real project checked out at `~/_my/rust/ptolemy`, as a sibling
of this tutorial's directory tree. The path in `Cargo.toml` is relative to this chapter.

## 12.1 Using the real crates

```toml
[dependencies]
# The real crates, from the sibling checkout at ~/_my/rust/ptolemy.
ptolemy-time = { path = "../../../ptolemy/crates/ptolemy-time" }
ptolemy-astro = { path = "../../../ptolemy/crates/ptolemy-astro" }
```

`../../../ptolemy` goes from `~/_my/rust/tutorial/ptolemy/ch12` up to `~/_my/rust/` and down into
`ptolemy`. Path dependencies are how you develop several crates side by side; for a published crate
it would be `ptolemy-astro = "0.1"` instead. The names in code use underscores: `ptolemy_astro`.

The public API you need is found by reading each `lib.rs` (chapter 8), or by running
`cargo doc --open -p ptolemy-astro` in the real project:

```rust
use ptolemy_astro::{Equatorial, StarCoord};                     // re-exported from lib.rs
use ptolemy_time::egyptian::{Egyptian, EgyptianMonth};          // a module
use ptolemy_time::{HistoricDateTime, JdError};
```

## 12.2 The program

```text
cargo run -p ch12 [YEAR]       positions of three stars in January of YEAR (default 134)
```

The data, as constants:

```rust
const STARS: [(&str, f64, f64); 3] = [
    ("Vega", 18.615_607_22, 38.782_993_11),
    ("Antares", 16.490_129_86, -26.431_946_08),
    ("Spica", 13.419_890_15, -11.161_244_91),
];

type StarPosition = (&'static str, f64, f64);
```

`type` introduces an alias; without it clippy complains that the return type is too complex
(`clippy::type_complexity`).

### The calculation

```rust
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
```

Walk through it:

1. `HistoricDateTime::new(...)` returns `Result<_, DateError>`; `?` converts the `DateError` into a
   `Box<dyn Error>` through `From` (chapter 4). The invalid dates, like February 30, are rejected
   here and nowhere later.
2. `StarCoord::new(epoch)` can fail with `JdError` if the epoch is before Julian Day 0.
3. The `.iter().map(...)` closure captures `sc` and `j2000` by reference. It returns `Ok((...))`; the
   `?` inside it needs the closure's error type to be known, which `collect::<Result<_, JdError>>()`
   pins down.
4. **Collecting `Result`s:** an iterator of `Result<T, E>` can be collected into
   `Result<Vec<T>, E>`: you get either all the values, or the first error, and iteration stops at
   it. (`Option` does the same.) The `_` lets the compiler infer the `Vec<StarPosition>`.
5. `.map_err(Into::into)` converts the `JdError` into `Box<dyn Error>`. A function path like
   `Into::into` can be used wherever a closure `|e| e.into()` is expected.
6. `Equatorial::from_hours` takes right ascension in hours, as in catalogues, and the library
   converts it to degrees.

### `main`

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let year = match std::env::args().nth(1) {
        Some(arg) => arg.parse()?,       // ParseIntError -> Box<dyn Error>
        None => 134,
    };

    let epoch = HistoricDateTime::new(year, 1, 1, 12, 0, 0)?;
    println!("Epoch {epoch} (JD {})", epoch.jd()?);
    println!("{:<8} {:>9} {:>9}", "star", "lambda", "beta");
    for (name, lon, lat) in star_positions(year)? {
        println!("{name:<8} {lon:>9.4} {lat:>9.4}");
    }
    // ...
}
```

- `std::env::args()` is an iterator of `String`; item 0 is the program name, so `.nth(1)` is the first
  argument, as an `Option`. A real tool would use the `clap` crate; for one argument this is
  enough.
- `arg.parse()?` infers `i32` from the use in `HistoricDateTime::new(year, ...)`.
- The format spec `{:<8}` left-aligns in 8 columns, `{:>9.4}` right-aligns with four decimals.
- `epoch.jd()?` returns the Julian Day as `f64`, or `JdError`.

```text
$ cargo run -p ch12
Epoch 134.01.01 12:00:00 (JD 1770002)
star        lambda      beta
Vega      259.4836   61.9675
Antares   223.8092   -4.3388
Spica     177.8897   -1.9520
```

These are the same numbers as the hand-written precession in chapter 9. Two independent
implementations (the tutorial's and the real crate's) agree to four decimals, which is the best
evidence that both are right.

Try other epochs, and the error paths:

```text
$ cargo run -p ch12 -- 2000       # almost no precession, J2000 positions in ecliptic coordinates
$ cargo run -p ch12 -- -5000      # before Julian Day 0
Error: BeforeEpoch
$ cargo run -p ch12 -- abc
Error: ParseIntError { kind: InvalidDigit }
```

(Both end with exit status 1. `main` prints `Error: ` followed by the `Debug` output of the error;
the first comes from `epoch.jd()?`, the second from `arg.parse()?`.)

### A reign and two eclipses

The second part uses `egyptian` (chapters 5, 6 and 7): two lunar eclipses of the Almagest, observed
in the 17th and 19th year of Hadrian, which Ptolemy used to determine a lunar parameter.

```rust
let payni: EgyptianMonth = "Payni".parse()?;        // FromStr: chapter 6
let choiak: EgyptianMonth = "Choiak".parse()?;
let first = Egyptian::new("Hadrianus", 17, payni, 20, 23, 15)?;
let second = Egyptian::new("Hadrianus", 19, choiak, 2, 23, 0)?;
println!("\nBetween the eclipses: {}", first.delta_duration(&second));
println!("Backwards:            {}", second.delta_duration(&first));
```

```text
Between the eclipses: + 1 years 166 days 23 hours 45 minutes 00 seconds
Backwards:            - 1 years 166 days 23 hours 45 minutes 00 seconds
```

Everything seen before shows up: `Egyptian::new` validates (unknown reign, day 31, hour 25 are
`EgyptianError`s), a `&'static str` reign name is stored inside, the `Duration` has a `Display` and a
sign, and swapping the arguments negates the result, because `Duration` arithmetic is antisymmetric
(chapter 6). The Egyptian calendar has twelve months of 30 days and 5 epagomenal days, which is why
no leap-year logic is needed to subtract two Egyptian dates.

## 12.3 Tests

```rust
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
```

One golden-style test against the output of the real example, and one test of the error path.
Run them with `cargo test -p ch12`.

## 12.4 Workspace tooling

Before every commit, in the tutorial workspace or in the real project:

```text
cargo fmt --all --check                          formatting, as rustfmt prescribes
cargo clippy --workspace --all-targets -- -D warnings   lints; warnings become errors
cargo test --workspace                           unit, integration and doc tests
cargo doc --workspace --no-deps --open           the API documentation in a browser
cargo build --release                            optimised build (to target/release/)
```

All of these are clean for every chapter of this tutorial, and for the real project.

`cargo clippy` is worth its own mention: it found real issues while the tutorial was written:
dead code, a needlessly complex return type (chapter 12 above), manual re-implementations of
standard functions. Read its suggestions, they teach idiomatic Rust.

## 12.5 What next?

The real project has plans which are good exercises (see `PLAN.md`, phase 9):

- **`ptolemy-cli`**: a command-line tool for star position and precession. Use `clap` for arguments
  and `assert_cmd` to test the binary. The program of this chapter is its prototype.
- **`ptolemy-gui`**: a graphical front end. The plan insists on keeping logic in the core crates
  and testing view-models without a window.
- The *open question* in the plan: the conversion-quality gradient was ported unchanged from the
  Python version and looks wrong. A characterisation test, then either keep it or replace it by the
  correct derivative of longitude with respect to right ascension, is a nice numerical task
  (chapters 9 and 11).

Other directions from here:

- *The Rust Programming Language* ("the book") and *Rust by Example*, for everything this tutorial
  skipped: `Rc`/`Arc`, `RefCell`, threads and `async`, macros, `unsafe`, FFI.
- *Astronomical Algorithms* by Jean Meeus, the source of the formulas in the project.
- Ptolemy's *Almagest*, of course.

## Exercises

1. Add a second argument: the star catalogue to read, as lines `name ra_hours dec`, using
   `parse_line` from chapter 5.
2. Print the table for several epochs given as `cargo run -p ch12 -- 134 1000 2000` using a loop
   over `std::env::args().skip(1)`, with an error message per bad argument but no early exit.
3. Add a `--json` flag that prints the table as JSON with `serde_json::json!`.
4. Use the batch API of `ptolemy_astro::batch` (chapter 11) to convert all three stars with one call
   instead of the loop.
5. Write the tests for the Egyptian part of `main`: extract it to a function that returns the two
   `Duration`s, and check `a == -b`.
6. Make `star_positions` return a custom `enum AppError` (with `#[from]` variants) instead of
   `Box<dyn Error>`, and compare the two designs.

## Summary

- A path dependency uses a sibling crate; its public API is what `lib.rs` re-exports.
- `Box<dyn Error>` plus `?` is the simplest error handling for applications; iterators of `Result`
  collect into a `Result` of a collection.
- Rust's compile-time checking, tests and tooling turn a port from Python into something you
  can trust: the types removed whole classes of bugs, and the tests pinned the numbers.
- You have now seen the language from `println!` to a multi-crate workspace. Keep the real project as a
  reference, and write your own.

Previous: [Chapter 11](../ch11/README.md) | Back to the [tutorial index](../README.md)
