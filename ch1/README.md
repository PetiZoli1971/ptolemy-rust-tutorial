# Chapter 1: Hello, Ptolemy

Claudius Ptolemy's *Almagest* (about 150 AD) dates every observation in the Egyptian calendar,
while modern star catalogues use the Julian Day. This chapter writes the first small program of the
project, a Julian Day calculator, and uses it to tour Cargo and Rust's primitive types.

**You will learn**

- creating, building and running a Cargo project, and how the `ptolemy` workspace is organised
- `println!` and format specifiers
- `let`, mutability, shadowing and `const`
- integers, floats, `bool`, tuples and arrays, and numeric conversions (`as`, `From`)
- writing a function and a first unit test

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch1` from the tutorial root.

## 1.1 The project at a glance

The real project is a Cargo *workspace*, a set of crates built together:

```text
ptolemy/
  Cargo.toml            workspace manifest
  crates/
    ptolemy-time/       dates, Julian Day, durations, Egyptian calendar
    ptolemy-astro/      solar/lunar parameters, precession, star coordinates
  fixtures/golden.json  reference values from the Python original
  tools/gen_golden.py   regenerates the fixture
```

From `~/_my/rust/ptolemy` you can already try the everyday commands:

```sh
cargo build --workspace                       # compile every crate
cargo test --workspace                        # run every test
cargo run -p ptolemy-astro --example almagest_stars
```

The last command prints the ecliptic positions of three stars at Ptolemy's epoch. At the end of the
tutorial you will understand every line behind that output.

> **Cargo in one paragraph.** `cargo new` creates a package with a `Cargo.toml` manifest and
> `src/main.rs` (a binary) or `src/lib.rs` (a library). `cargo build` compiles into `target/`,
> `cargo run` builds and runs, `cargo test` runs tests, `cargo check` type-checks quickly without
> producing a binary. `cargo build --release` turns on optimisations.

## 1.2 Hello, world

```rust
fn main() {
    println!("Hello, Ptolemy!");
}
```

`fn main()` is the entry point. `println!` ends in `!` because it is a *macro*, not a function; it
checks the format string at compile time. Placeholders are `{}` (uses `Display`) and `{:?}` (uses
`Debug`); a variable name can go inside the braces, as in `{jd}`.

## 1.3 A first calculation

The Julian Day of a Gregorian date (Jean Meeus, *Astronomical Algorithms*, ch. 7):

```rust
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
    (365.25 * f64::from(y + 4716)).floor() + (30.6001 * f64::from(m + 1)).floor() + day + b
        - 1524.5
}
```

Things to notice:

- `const` items need a type annotation and are evaluated at compile time. By convention they are
  `SCREAMING_SNAKE_CASE`. Underscores in numeric literals (`2_451_545.0`) are ignored.
- A function signature spells out all parameter and return types: `fn name(a: T) -> R`.
- The function body is an *expression*: the last line has no semicolon, so its value is returned.
  There is no `return` keyword needed.
- `if` is an expression too. Here it yields a tuple, which is destructured straight into `(y, m)`.
- Rust never converts numbers implicitly. `f64::from(y)` is a lossless conversion from `i32`;
  using `y` where an `f64` is expected would not compile.

Using it:

```rust
let jd = julian_day(2000, 1, 1.5);
println!("J2000.0 is Julian Day {jd}");
assert!((jd - J2000).abs() < 1e-9);
```

```text
J2000.0 is Julian Day 2451545
```

`assert!` panics (aborts the thread with a message) if the condition is false. Never compare floats
with `==` unless you know the arithmetic is exact; compare against a tolerance, as above.

## 1.4 Variables: mutability and shadowing

Bindings are immutable by default. `mut` opts in to changes. *Shadowing* creates a new binding with
the same name, which may even have a different type:

```rust
let sputnik = julian_day(1957, 10, 4.81);   // Meeus example 7.a
println!("Sputnik 1: JD {sputnik:.2}");

let t = sputnik - J2000;          // days
let t = t / DAYS_PER_CENTURY;     // now centuries: a new variable named `t`
println!("T = {t:.9} Julian centuries since J2000.0");
```

```text
Sputnik 1: JD 2436116.31
T = -0.422414511 Julian centuries since J2000.0
```

`{sputnik:.2}` means "format with 2 decimals". The same `T` (Julian centuries since J2000.0) is the
time argument of almost every formula in chapter 9.

```rust
let mut counted = 0;
counted += 1;     // allowed only because of `mut`
```

## 1.5 Primitive types

| Kind | Types | Example |
|---|---|---|
| signed integers | `i8 i16 i32 i64 i128 isize` | `-4712_i32` |
| unsigned integers | `u8 u16 u32 u64 u128 usize` | `13_u8` |
| floats | `f32 f64` | `18.615_607_22_f64` |
| boolean | `bool` | `true` |
| character | `char` (a Unicode scalar, 4 bytes) | `'α'` |
| unit | `()` | the value of a block with no result |

Integer literals default to `i32`, float literals to `f64`. `f64` is the right choice for angles
and Julian Days; the project uses `u8` for months and days, because a month needs no more than 8
bits.

### Converting

```rust
let ra_hours: f64 = 18.615_607_22; // Vega's right ascension
let ra_degrees = ra_hours * 15.0;  // 24 h = 360 degrees
let whole_hours = ra_hours as u8;  // `as` truncates towards zero
let minutes = ((ra_hours - f64::from(whole_hours)) * 60.0) as u8;
println!("Vega: RA {ra_hours} h = {ra_degrees:.4} deg = {whole_hours}h {minutes:02}m");
```

```text
Vega: RA 18.61560722 h = 279.2341 deg = 18h 36m
```

- `x as T` always compiles but can lose information (truncation, wrap-around, saturation of floats).
- `T::from(x)` and `x.into()` exist only for conversions that cannot lose information. Prefer them
  whenever they compile.
- `T::try_from(x)` returns a `Result` for conversions that can fail.

### Integers have a width

```rust
let secs_per_year: i32 = 365 * 86_400;
println!("i32::MAX = {}, seconds per year = {secs_per_year}", i32::MAX);
println!("6745 years in i32: {:?}", secs_per_year.checked_mul(6745)); // None means overflow
let total_secs = i64::from(secs_per_year) * 6745;
println!("6745 years in i64: {total_secs}");
```

```text
i32::MAX = 2147483647, seconds per year = 31536000
6745 years in i32: None
6745 years in i64: 212710320000
```

6745 is the number of 365-day years between the Julian Day epoch (-4712) and 2028 (the real
`HistoricDateTime::jd_duration` returns exactly that). In a debug build,
`secs_per_year * 6745` would panic with "attempt to multiply with overflow"; in a release build it
would silently wrap around. That is why the real `Duration` stores an `i64`. The family of
`checked_*`, `wrapping_*`, `saturating_*` and `overflowing_*` methods lets you choose the behaviour.

### Tuples and arrays

```rust
let epoch: (i32, u8, u8) = (134, 1, 1);
println!("Ptolemy's epoch: {}-{:02}-{:02}", epoch.0, epoch.1, epoch.2);

let names = ["Vega", "Antares", "Spica"];          // [&str; 3]
let ra = [18.615_607_22, 16.490_129_86, 13.419_890_15];
let mut counted = 0;
for (name, h) in names.iter().zip(ra) {
    println!("{name:<8} {:>9.4} deg", h * 15.0);
    counted += 1;
}
println!("{counted} stars");
```

```text
Ptolemy's epoch: 134-01-01
Vega      279.2341 deg
Antares   247.3519 deg
Spica     201.2984 deg
3 stars
```

- A tuple groups values of different types; fields are accessed as `.0`, `.1`, ...
- An array `[T; N]` has a fixed length known at compile time and lives on the stack. Chapter 7
  covers `Vec<T>`, the growable alternative.
- `{:<8}` pads on the right to width 8, `{:>9.4}` right-aligns in width 9 with 4 decimals,
  `{:02}` pads with zeros.

## 1.6 Your first test

At the bottom of `main.rs`:

```rust
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
```

`#[cfg(test)]` compiles the module only for `cargo test`. `use super::*;` brings the parent
module's items into scope. Each `#[test]` function is a test; it fails if it panics.

```sh
cargo test -p ch1
```

## 1.7 Limits of this first version

The formula above is only correct for Gregorian dates. For a date in Ptolemy's era you must use the
Julian calendar, where the correction term `b` is `0`. It also uses `f64` for everything, which
makes the "does this date exist?" question awkward. The next chapters fix both: chapter 2 builds
the calendar rules, chapter 4 a validated date type, and chapter 9 the Julian-calendar branch.

**In the real project:** [`historic_datetime.rs`](../../../ptolemy/crates/ptolemy-time/src/historic_datetime.rs)
computes the Julian Day Number in exact integer arithmetic (`julian_day_number`) and converts to
`f64` only at the very end. The README's "Differences from the Python original" section lists the
bug the float formula hides: using the unadjusted year for the century term gives wrong answers for
January and February of years such as 1900.

## Exercises

1. Run `cargo run -p ch1`. Change the date to your birthday and print its Julian Day.
2. Compute the number of days between 1957-10-04.81 and 2000-01-01.5 using `julian_day` and print
   it with `{:.2}`. (Hint: subtract two `f64` values.)
3. Make `main` print the right ascension of Spica as hours, minutes and seconds. You need one more
   conversion step beyond the minutes.
4. Declare `let x: u8 = 255;` and `let y = x + 1;`. What does the compiler say? Replace the
   addition with `x.checked_add(1)`, `x.wrapping_add(1)` and `x.saturating_add(1)` and compare.
5. Bonus: write a test that asserts the Julian Day of 1987-01-27.0 is 2446822.5 (Meeus 7.a).

## Summary

- Cargo builds, runs and tests packages; the project is a workspace of two library crates.
- Bindings are immutable unless marked `mut`; shadowing makes a new binding.
- Numbers never convert implicitly: use `From`, `TryFrom`, or `as` knowingly.
- A block's last expression, without `;`, is its value.
- Tests are ordinary functions marked `#[test]`.

Next: [Chapter 2: Calendar rules](../ch2/README.md)
