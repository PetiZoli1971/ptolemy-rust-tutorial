# Chapter 4: Error handling

`HistoricDateTime::new(1900, 2, 29, ...)` must fail: 1900 was not a leap year. So must month 13,
hour 25, and every day between 1582-10-05 and 1582-10-14. Python raised exceptions, or in some
places silently returned `-1` or `None`. Rust has no exceptions; failures are values in the return
type, and the compiler makes sure you look at them.

**You will learn**

- `Option` versus `Result`, and when to use which
- `match`, `if let`, `unwrap_or`, `expect`, `map_err`, `ok_or`
- the `?` operator and early return
- defining error enums, by hand and with `thiserror`
- `From` and `#[from]` for converting between error types
- `let ... else`
- `main` returning `Result`, and `Box<dyn Error>`
- "parse, don't validate": a type whose constructor enforces its invariants

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch4` (it ends with an error on purpose,
exit code 1).

## 4.1 Panic, `Option`, `Result`

| Mechanism | Meaning | Use for |
|---|---|---|
| `panic!`, `unwrap()`, indexing out of bounds | unrecoverable bug: the thread unwinds | broken invariants, tests, examples |
| `Option<T>` | a value may legitimately be absent | lookups, "not found" |
| `Result<T, E>` | an operation may fail, with a reason | I/O, parsing, validation of input |

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Rule of thumb for library code: never panic because of bad *input*; return a `Result`. Panic only
for bugs in the caller or in the library, and document it (see `# Panics` in chapter 10).

## 4.2 An error type

An error type is usually an enum with one variant per failure, carrying the offending data:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
enum ManualJdError {
    BeforeEpoch,
}
```

To be a proper error it needs `Display` (the message for humans) and the `Error` marker trait:

```rust
use std::error::Error;
use std::fmt;

impl fmt::Display for ManualJdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManualJdError::BeforeEpoch => {
                f.write_str("Julian Day is only defined from -4712-01-01 12:00:00")
            }
        }
    }
}

impl Error for ManualJdError {}
```

That is a lot of ceremony. The `thiserror` crate generates it from attributes:

```rust
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
enum DateError {
    #[error("month must be between 1 and 12, got {0}")]
    Month(u8),
    #[error("day {day} is not valid for {year}-{month:02}")]
    Day { year: i32, month: u8, day: u8 },
    #[error("dates between 1582-10-05 and 1582-10-14 do not exist")]
    SkippedByReform,
    #[error("hour must be between 0 and 23, got {0}")]
    Hour(u8),
    #[error("minute must be between 0 and 59, got {0}")]
    Minute(u8),
    #[error("second must be between 0 and 59, got {0}")]
    Second(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
enum JdError {
    #[error("Julian Day is only defined from -4712-01-01 12:00:00")]
    BeforeEpoch,
}
```

`{0}` refers to the first unnamed field and `{day}` to a named one. `thiserror` is a compile-time
helper, so the generated code is the same as the handwritten one, and adds nothing at run time.
Add it with `cargo add thiserror` or by hand in `Cargo.toml`:

```toml
[dependencies]
thiserror = "2"
```

Why *two* error types, `DateError` and `JdError`? They fail at different moments: creating a date
with impossible fields, versus converting a valid date that lies before the Julian Day epoch. A
caller of `jd()` should not have to handle "month 13".

## 4.3 A constructor that cannot lie

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct HistoricDateTime {
    year: i32, month: u8, day: u8, hour: u8, minute: u8, sec: u8,
}

impl HistoricDateTime {
    fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, sec: u8)
        -> Result<Self, DateError>
    {
        // `ok_or` turns an `Option` into a `Result`; `?` returns early on `Err`.
        let max_day = days_in_month(year, month).ok_or(DateError::Month(month))?;
        if year == 1582 && month == 10 && (5..15).contains(&day) {
            return Err(DateError::SkippedByReform);
        }
        if !(1..=max_day).contains(&day) {
            return Err(DateError::Day { year, month, day });
        }
        if hour >= 24 { return Err(DateError::Hour(hour)); }
        if minute >= 60 { return Err(DateError::Minute(minute)); }
        if sec >= 60 { return Err(DateError::Second(sec)); }
        Ok(Self { year, month, day, hour, minute, sec })
    }
}
```

The fields are private and `new` is the only constructor, so **every** `HistoricDateTime` in the
program is a valid date. Later code never has to re-check. This is the idea behind "parse, don't
validate".

Callers have to deal with the result:

```rust
// 1. Match on it.
match HistoricDateTime::new(1582, 10, 10, 0, 0, 0) {
    Ok(d) => println!("valid: {d:?}"),
    Err(e) => println!("error: {e}"),
}

// 2. Look inside the error.
let err = HistoricDateTime::new(1900, 2, 29, 0, 0, 0).unwrap_err();
println!("{err}   ({err:?})");
if let DateError::Day { year, .. } = err {
    println!("the offending year was {year}");
}
```

```text
error: dates between 1582-10-05 and 1582-10-14 do not exist
day 29 is not valid for 1900-02   (Day { year: 1900, month: 2, day: 29 })
the offending year was 1900
```

`{e}` uses `Display` (the message), `{err:?}` uses `Debug` (the structure).

Frequently used `Result` methods:

| Method | Effect |
|---|---|
| `unwrap()` / `expect("why")` | the `Ok` value, or panic. Fine in tests; elsewhere only if "cannot happen" |
| `unwrap_or(v)` / `unwrap_or_else(f)` | the `Ok` value or a fallback |
| `map(f)` / `map_err(f)` | transform the `Ok` / the `Err` |
| `ok()` | drop the error and get an `Option` |
| `is_ok()` / `is_err()` | test |
| `unwrap_err()` | the error, or panic (handy in tests) |

## 4.4 The `?` operator

`?` after an expression of type `Result<T, E>` means: if it is `Ok(v)`, evaluate to `v`; if it is
`Err(e)`, return `Err(From::from(e))` from the current function.

```rust
fn jd(&self) -> Result<f64, JdError> {
    let time_of_day =
        i64::from(self.hour) * 3_600 + i64::from(self.minute) * 60 + i64::from(self.sec);
    let secs = self.julian_day_number() * SECS_PER_DAY + time_of_day - SECS_PER_DAY / 2;
    if secs < 0 {
        Err(JdError::BeforeEpoch)
    } else {
        Ok(secs as f64 / SECS_PER_DAY as f64)
    }
}
```

The Julian Day Number is computed with integer arithmetic (the exact formula is
`(1461 * (y + 4716)).div_euclid(4) + (306001 * (m + 1)).div_euclid(10000) + day + b - 1524`).
`div_euclid` rounds towards negative infinity, which is what the formula needs for dates before
year 0; plain `/` rounds towards zero and would give wrong results. Seconds since the epoch are
kept as `i64` and divided by 86400 only at the end, so no floating-point error creeps in.

```rust
let j2000 = HistoricDateTime::new(2000, 1, 1, 12, 0, 0).unwrap();
assert_eq!(j2000.jd(), Ok(2_451_545.0));

let before = HistoricDateTime::new(-4712, 1, 1, 11, 59, 59).unwrap();
assert_eq!(before.jd(), Err(JdError::BeforeEpoch));
```

## 4.5 Combining error types with `From`

Parsing a date from text can fail in three ways: wrong shape, a non-numeric field, or an invalid
date. One enum with `#[from]` conversions lets `?` do all the work:

```rust
use std::num::ParseIntError;

#[derive(Debug, Error, PartialEq, Eq)]
enum ParseError {
    #[error("expected YYYY.MM.DD, got {0:?}")]
    Format(String),
    #[error("not a number: {0}")]
    Number(#[from] ParseIntError),
    #[error(transparent)]
    Date(#[from] DateError),
}

/// Parses `YYYY.MM.DD` (the separator is `.` so that negative years work).
fn parse_date(s: &str) -> Result<HistoricDateTime, ParseError> {
    let mut parts = s.split('.');
    let (Some(y), Some(m), Some(d), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ParseError::Format(s.to_owned()));
    };
    // Each `?` converts the error with `From`.
    Ok(HistoricDateTime::new(y.parse()?, m.parse()?, d.parse()?, 0, 0, 0)?)
}
```

- `#[from]` generates `impl From<ParseIntError> for ParseError`. That is what `?` calls
  when `y.parse()` fails, so no `map_err` is needed.
- `y.parse()` has no turbofish: the compiler infers `i32` for `y` and `u8` for `m` and `d` from
  the signature of `new`.
- `#[error(transparent)]` forwards `Display` and `source()` to the inner error, so the message of
  a `DateError` is not wrapped in extra text.
- `let ... else` destructures a pattern and runs the `else` block (which must diverge: `return`,
  `break`, `panic!`) if it does not match. The pattern `(Some, Some, Some, None)` ensures there
  are exactly three parts.

```text
2000.01.01   -> HistoricDateTime { year: 2000, month: 1, day: 1, hour: 0, minute: 0, sec: 0 }, JD Ok(2451544.5)
2000.13.01   -> month must be between 1 and 12, got 13
20x0.01.01   -> not a number: invalid digit found in string
2000.01      -> expected YYYY.MM.DD, got "2000.01"
-4712.01.01  -> HistoricDateTime { year: -4712, ... }, JD Err(BeforeEpoch)
```

A last layer, `AppError`, shows how errors of different layers compose:

```rust
#[derive(Debug, Error)]
enum AppError {
    #[error(transparent)] Parse(#[from] ParseError),
    #[error(transparent)] Jd(#[from] JdError),
}

fn days_between(from: &str, to: &str) -> Result<f64, AppError> {
    let a = parse_date(from)?.jd()?;   // ParseError or JdError, both converted
    let b = parse_date(to)?.jd()?;
    Ok(b - a)
}
```

```text
days from 1957.10.04 to 2000.01.01: 15429.0
```

The real `ptolemy_astro::batch::BatchError` does exactly this with `#[from] JdError`.

### Library errors versus application errors

- Libraries return specific enums (`DateError`, `JdError`) so callers can `match` on them.
- Applications often just want to print the error and exit. `Box<dyn Error>` can hold any error;
  `?` converts into it automatically.

```rust
fn main() -> Result<(), Box<dyn Error>> {
    // ...
    let last = parse_date("2000.02.30")?;   // 30 February: fails
    println!("not reached: {last:?}");
    Ok(())
}
```

```text
Error: Date(Day { year: 2000, month: 2, day: 30 })
```

When `main` returns `Err`, the process prints it using `Debug` and exits with status 1.

## 4.6 Which one do I return?

```text
Can the absence be explained by "nothing found"?     -> Option<T>
Does the caller need to know *why* it failed?        -> Result<T, MyError>
Is it a bug if this happens?                         -> panic!/assert!/expect
```

`days_in_month` (chapter 2) returns an `Option` because there is only one reason for failure.
`HistoricDateTime::new` returns a `Result` because there are six.

**In the real project:** see [`error.rs`](../../../ptolemy/crates/ptolemy-time/src/error.rs) for
`DateError`, `JdError` and `DurationError`, and
[`historic_datetime.rs`](../../../ptolemy/crates/ptolemy-time/src/historic_datetime.rs) for the
constructor. The `egyptian` module defines `EgyptianError` and, for the month names, a
`FromStr` implementation that returns an error where Python returned `-1` (chapter 6).

## Exercises

1. Add a `fn parse_time(s: &str) -> Result<(u8, u8, u8), ParseError>` for `"HH:MM:SS"`, using
   `#[from]` for the number errors. Reuse `DateError::Hour`/`Minute`/`Second` for out-of-range
   values.
2. Extend `parse_date` to accept `"YYYY.MM.DD HH:MM:SS"`.
3. In the tests, check that every day from the 5th to the 14th of October 1582 fails and the 4th and
   15th succeed (the crate already does; find where).
4. Make `days_between` return `Result<f64, Box<dyn Error>>` instead of `AppError`. What do you lose
   (hint: try to `match` on the result)?
5. Replace `unwrap_or(f64::NAN)` in the demo by a `match` that prints the error message instead.
6. Add `#[derive(Hash)]` to `DateError`; why is it impossible to do the same for an error type that
   contains an `f64`? (Hint: which of `Eq` and `Hash` does `f64` implement?)

## Summary

- Failure is part of the type: `Option<T>` for absence, `Result<T, E>` for failure with a reason.
- `?` returns early and converts errors with `From`; `thiserror` removes the boilerplate.
- A constructor returning `Result` plus private fields gives types that cannot be invalid.
- `main` can return `Result<(), Box<dyn Error>>`.

Previous: [Chapter 3](../ch3/README.md) | Next: [Chapter 5: Ownership, borrowing, lifetimes](../ch5/README.md)
