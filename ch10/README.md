# Chapter 10: Testing

Ptolemy is a *port*: the Rust code must give the same numbers as the Python original (except
where the original was wrong). That makes testing the main safeguard, and Rust has first-class
support for it: no framework to install, `cargo test` finds and runs everything.

**You will learn**

- unit tests, integration tests and doctests, and where each lives
- `assert!`, `assert_eq!`, `assert_ne!`, custom messages, and `approx` for floats
- `#[should_panic]`, `#[ignore]`, tests that return `Result`
- golden-file tests with `include_str!` and `serde_json`
- property-based tests with `proptest`
- running a subset of tests, seeing output, test-driven development
- how the real project tests itself

Code: [`src/lib.rs`](src/lib.rs), [`tests/golden.rs`](tests/golden.rs),
[`tests/golden.json`](tests/golden.json), [`tests/properties.rs`](tests/properties.rs).
Run it with `cargo test -p ch10`. (This chapter is a library, so there is no `cargo run`.)

## 10.1 The library under test

`ch10` is a deliberately tiny library: `is_leap_year`, `julian_day_number` (integer arithmetic),
`julian_day` (decimal), `normalize_deg` and `angle_diff`, all from earlier chapters. Its dependencies
are only for tests:

```toml
[dev-dependencies]
approx.workspace = true
proptest.workspace = true
serde_json.workspace = true
```

`[dev-dependencies]` are compiled for tests, examples and benchmarks, but are not dependencies of
the library for its users.

## 10.2 Unit tests

A unit test is a function marked `#[test]`, in a `#[cfg(test)]` module of the same file.
It passes when it returns without panicking.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn leap_years() {
        assert!(is_leap_year(-4));
        assert!(!is_leap_year(1700));
    }

    #[test]
    fn julian_day_of_j2000() {
        assert_abs_diff_eq!(julian_day(2000, 1, 1, 12, 0, 0), 2_451_545.0, epsilon = 1e-9);
    }
}
```

| Macro | Use |
|---|---|
| `assert!(cond)` | a boolean. `assert!(cond, "message {x}")` adds a message |
| `assert_eq!(a, b)` / `assert_ne!(a, b)` | equality; prints both sides when it fails, so prefer them to `assert!(a == b)` |
| `assert_abs_diff_eq!(a, b, epsilon = e)` | floats within an absolute tolerance (`approx`) |
| `assert_relative_eq!(a, b, max_relative = r)` | floats within a relative tolerance |
| `debug_assert!` | checked only in debug builds |

`assert_eq!` on floats needs `PartialEq`, which `f64` has, but exact comparison of computed values is
nearly always wrong (chapter 9). Use `approx`.

The unit tests may call private functions, because the test module is a child of the module it
tests (chapter 8).

### Expected panics

The library says in its docs that `julian_day_number` panics on a bad month. Test it:

```rust
#[test]
#[should_panic(expected = "month must be between 1 and 12")]
fn invalid_month_panics() {
    julian_day_number(2000, 0, 1);
}
```

`expected` is a substring of the panic message; without it, *any* panic would make the test
pass. The function documents the contract in a `# Panics` section, which is the convention
for functions that can panic:

```rust
/// Julian Day Number of the noon of a calendar day, in integer arithmetic.
///
/// # Panics
///
/// Panics if `month` is not in `1..=12`.
```

(The real `HistoricDateTime::new` returns a `Result` instead; chapter 4 explains when to choose
which.)

### Tests that return `Result`

```rust
#[test]
fn returns_result() -> Result<(), String> {
    let n: i64 = "2451545".parse::<i64>().map_err(|e| e.to_string())?;
    assert_eq!(julian_day_number(2000, 1, 1), n);
    Ok(())
}
```

With `-> Result<(), E>` the `?` operator works inside the test, and an `Err` fails it. The real
crates' tests use `Result<(), Box<dyn Error>>` to avoid `unwrap()` everywhere.

### Ignored tests

```rust
#[test]
#[ignore = "slow in debug builds; run with `cargo test -- --ignored`"]
fn consecutive_days_for_a_long_range() { /* ... */ }
```

Ignored tests are listed but not run, unless you ask for them: `cargo test -- --ignored` runs only
those; `cargo test -- --include-ignored` runs all.

## 10.3 Doctests

Code blocks in documentation comments are compiled and run as tests:

```rust
/// Julian rule up to and including 1582, Gregorian rule afterwards.
///
/// ```
/// use ch10::is_leap_year;
/// assert!(is_leap_year(2000));
/// assert!(!is_leap_year(1900));
/// assert!(is_leap_year(1500)); // Julian calendar
/// ```
pub fn is_leap_year(year: i32) -> bool { /* ... */ }
```

A doctest uses only the public API, exactly as a user would, and guarantees that the examples in the
documentation work. Block attributes: ` ```should_panic `, ` ```no_run ` (compile only),
` ```ignore `, ` ```text ` (not Rust). Lines starting with `# ` are compiled but hidden in the
rendered documentation.

The test run lists them with the line numbers:

```text
   Doc-tests ch10
test ch10/src/lib.rs - is_leap_year (line 14) ... ok
test ch10/src/lib.rs - (line 5) ... ok
test ch10/src/lib.rs - julian_day_number (line 38) ... ok
test ch10/src/lib.rs - julian_day_number (line 34) ... ok
test ch10/src/lib.rs - normalize_deg (line 71) ... ok
```

## 10.4 Integration tests and golden files

Files in `tests/` are separate crates which depend on your library and can only see its public API.
Each file is a test binary.

A **golden test** compares results with reference values produced by a trusted source. Here
[`tests/golden.json`](tests/golden.json) holds Julian days known from published examples:

```json
{
  "jd": [
    { "date": [-4712, 1, 1, 12, 0, 0], "jd": 0.0 },
    { "date": [2000, 1, 1, 12, 0, 0], "jd": 2451545.0 },
    { "date": [1957, 10, 4, 19, 26, 24], "jd": 2436116.31 },
    { "date": [-1000, 2, 29, 0, 0, 0], "jd": 1355866.5 },
    { "date": [1900, 1, 15, 0, 0, 0], "jd": 2415034.5 }
  ]
}
```

```rust
use approx::assert_abs_diff_eq;
use ch10::julian_day;
use serde_json::Value;

const GOLDEN: &str = include_str!("golden.json");

#[test]
fn julian_day_matches_golden_values() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("golden.json must be valid JSON");
    let cases = golden["jd"].as_array().expect("`jd` must be an array");
    assert!(!cases.is_empty());

    for case in cases {
        let d: Vec<i64> = case["date"].as_array().unwrap()
            .iter().map(|x| x.as_i64().unwrap()).collect();
        let expected = case["jd"].as_f64().unwrap();
        let got = julian_day(d[0] as i32, d[1] as u8, d[2] as u8, d[3] as u8, d[4] as u8, d[5] as u8);
        assert_abs_diff_eq!(got, expected, epsilon = 1e-6);
    }
}
```

- `include_str!` embeds the file's text in the test binary **at compile time**, so the test does not
  depend on the working directory. A missing file is a compile error.
- `serde_json::Value` is a dynamically typed JSON tree. For bigger fixtures you would define structs
  and use `serde::Deserialize` instead.
- `assert!(!cases.is_empty())` guards against the classic silent failure of a loop test: an empty
  or wrongly-named list makes every loop pass vacuously.
- `-1000-02-29` appears among the cases because the Julian calendar's leap-year rule, which holds
  for all years up to 1582, makes year -1000 a leap year. The January 1900 entry is the Python bug case
  from chapter 9.

A second integration test checks a *relationship*: the day after 4 October 1582 is
15 October, so JD must increase by exactly 1:

```rust
#[test]
fn julian_day_is_continuous_over_the_gregorian_reform() {
    let last_julian = julian_day(1582, 10, 4, 0, 0, 0);
    let first_gregorian = julian_day(1582, 10, 15, 0, 0, 0);
    assert_abs_diff_eq!(first_gregorian - last_julian, 1.0, epsilon = 1e-9);
}
```

Where do the golden values come from? In the real project they are not typed by hand, but dumped
from the original Python package by the script
[`tools/gen_golden.py`](../../../ptolemy/tools/gen_golden.py) into
[`fixtures/golden.json`](../../../ptolemy/fixtures/golden.json), and compared with by
[`crates/ptolemy-astro/tests/golden.rs`](../../../ptolemy/crates/ptolemy-astro/tests/golden.rs).
The script only records code paths that are correct in Python and patches the known `mean_obliquity`
bug in a subclass: the golden file is *evidence*, not an authority.

## 10.5 Property-based tests

An example test checks one input. A **property** test states a law and lets `proptest` try
hundreds of random inputs, then *shrinks* a failing input to a minimal counter-example.

```rust
use ch10::{angle_diff, julian_day_number, normalize_deg};
use proptest::prelude::*;

proptest! {
    #[test]
    fn normalize_is_always_in_range(x in -1.0e6f64..1.0e6) {
        let n = normalize_deg(x);
        prop_assert!((0.0..360.0).contains(&n), "{n}");
    }

    #[test]
    fn angle_diff_is_in_range_and_antisymmetric(a in 0.0f64..360.0, b in 0.0f64..360.0) {
        let d = angle_diff(a, b);
        prop_assert!((-180.0..=180.0).contains(&d));
        // The sign flips, except exactly at the +-180 boundary.
        prop_assert!((d + angle_diff(b, a)).abs() < 1e-9 || (d.abs() - 180.0).abs() < 1e-9);
    }

    #[test]
    fn consecutive_days_differ_by_one(year in -4000i32..3000, month in 1u8..=12, day in 1u8..=27) {
        // The ten skipped days of 1582 would break the property.
        prop_assume!(!(year == 1582 && month == 10));
        let a = julian_day_number(year, month, day);
        let b = julian_day_number(year, month, day + 1);
        prop_assert_eq!(b - a, 1);
    }
}
```

- `name in strategy` says how to generate the argument: ranges, `any::<u8>()`, vectors, tuples...
- `prop_assert!` / `prop_assert_eq!` report failures so that proptest can shrink them.
- `prop_assume!` discards inputs that do not satisfy a precondition (the ten days that do not exist
  in the 1582 reform month).

Good properties for this code base:

| Law | Example |
|---|---|
| range | `normalize_deg(x)` is in `[0, 360)` |
| round trip | ecliptic to equatorial and back returns the input (chapter 9) |
| idempotence | `normalize_deg(normalize_deg(x)) == normalize_deg(x)` |
| antisymmetry | `a - b == -(b - a)` for durations |
| consistency | a batch function equals the scalar one element by element (chapter 11) |
| monotonicity | later dates have larger Julian days |

Without the `prop_assume!` the third property fails for `(1582, 10, 4)`; shrinking reports that
minimal input. When a property fails, proptest saves the failing seed in a
`proptest-regressions/` file, so it is re-run from then on; commit those files.

## 10.6 Running tests

```text
cargo test                         everything in the workspace
cargo test -p ch10                 one package
cargo test julian                  every test whose name contains "julian"
cargo test --test golden           one integration-test file (tests/golden.rs)
cargo test --lib                   unit tests only
cargo test --doc                   doctests only
cargo test -- --nocapture          show println! output of passing tests
cargo test -- --test-threads=1     run sequentially
cargo test -- --ignored            only the #[ignore]d tests
```

Everything before `--` is for cargo, everything after goes to the test binary. Tests run in
parallel threads by default, and their output is captured and shown only for failing tests.

The result of `cargo test -p ch10`:

```text
unittests src/lib.rs        4 passed; 0 failed; 1 ignored
tests/golden.rs             2 passed
tests/properties.rs         3 passed
Doc-tests ch10              5 passed
```

## 10.7 Test-driven development

TDD is a rhythm of three steps:

1. **Red**: write a failing test describing the behaviour you want.
2. **Green**: write the simplest code that passes.
3. **Refactor**: clean up, with the tests as a safety net.

For a port, a good order is: golden test first (what must the number be?), then tests for the
bugs of the original (`b - a` must not modify `b`; ordering uses the sign), then property tests
(which laws must hold?), and then the implementation.

Try it in the library: add a test for a function that does not exist yet.

```rust
#[test]
fn day_of_week_of_j2000_is_saturday() {
    assert_eq!(day_of_week(2000, 1, 1), 6);     // error[E0425]: cannot find function
}
```

The compile error is the "red"; then write `day_of_week` (exercise 1).

## 10.8 How the real project tests itself

| Where | What |
|---|---|
| unit tests inside `src/*.rs` | small private helpers |
| `crates/ptolemy-time/tests/` | `calendar`, `duration`, `egyptian`, `historic_datetime`: rules, edge cases, the bugs of the original |
| `crates/ptolemy-astro/tests/` | `astro_params`, `star_coord`, `batch`: Meeus examples, derivative against a numeric slope, batch equals scalar |
| `crates/ptolemy-astro/tests/golden.rs` | comparison with `fixtures/golden.json` produced by Python |
| doctests in `lib.rs` and modules | usage examples |
| `cargo clippy -- -D warnings`, `cargo fmt --check` | style and common mistakes, checked in the same way as tests |

Run the whole real suite with `cd ~/_my/rust/ptolemy && cargo test --workspace`.

## Exercises

1. Implement `pub fn day_of_week(year: i32, month: u8, day: u8) -> u8` (0 = Sunday ... 6 = Saturday)
   as `(julian_day_number(year, month, day) + 1).rem_euclid(7)`, make the red test above pass, and
   add a doctest.
2. Add another reference value to `golden.json` (for example the 2000-01-01 00:00 value
   2451544.5, or a date of your own checked with an independent calculator) and run the test.
3. Write a property that `julian_day` is strictly increasing in `(hour, minute, sec)` for a fixed
   date.
4. Break `normalize_deg` by removing the `>= 360.0` guard. Does the property test find it? Why
   (not)? What input range would find it?
5. Add `#[should_panic]` without `expected`, then change the code to panic for a different reason.
   Does the test still pass? What does that tell you about `expected`?
6. Use `cargo test -- --nocapture` with a `println!` in a test and compare the output with and
   without the flag.

## Summary

- `#[test]` functions in a `#[cfg(test)]` module test internals; files in `tests/` test the public
  API; doc comments test the examples.
- Compare floats with a tolerance, panics with `#[should_panic(expected = ...)]`, and use `Result`
  returning tests for `?`.
- Golden files tie the port to the original, property tests check laws over many inputs.
- `cargo test` has filters; run the whole workspace before every commit.

Previous: [Chapter 9](../ch9/README.md) | Next: [Chapter 11: External crates: ndarray and log](../ch11/README.md)
