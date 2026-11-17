# Chapter 2: Calendar rules

Before any date can be turned into a Julian Day, the program must know which dates exist. That
takes a handful of rules: which years are leap years, how long each month is, and which ten days
the Gregorian reform deleted. These are small pure functions, ideal for learning Rust's functions,
expressions and control flow.

**You will learn**

- functions, return values and the "last expression" rule
- `if`/`else` as an expression
- `match`, including alternatives (`|`), ranges and tuple patterns, and `matches!`
- `for`, `while` and `loop`, ranges, `break` with a value
- `Option<T>`, `Some`/`None`, `if let`, and the `?` operator on `Option`
- constant tables: arrays, slices, `&str` and `&'static str`

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch2`.

## 2.1 The leap-year rule

The calendar of the project is a *hybrid*: the Julian rule up to and including 1582, the Gregorian
rule afterwards.

| Years | Rule |
|---|---|
| up to 1582 | every year divisible by 4 |
| after 1582 | divisible by 4, except centuries, except centuries divisible by 400 |

```rust
/// Julian rule up to and including 1582, Gregorian rule afterwards.
fn is_leap_year(year: i32) -> bool {
    if year <= 1582 {
        year % 4 == 0
    } else {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }
}
```

- `///` starts a *doc comment* that `cargo doc` renders as documentation (chapter 10 tests the
  code inside them).
- The two branches of `if` have the same type (`bool`), so the whole `if` is the value of the
  function. No semicolons, no `return`.
- `%` is the remainder. For negative `i32` it keeps the sign of the dividend: `-4 % 4 == 0`,
  `-1 % 4 == -1`. That is all we need here because we only test for `== 0`.
- Years use astronomical numbering, so `-748` (749 BC) is a leap year in the Julian calendar.

```rust
for year in [-748, 1, 1500, 1582, 1700, 1900, 2000, 2024] {
    let kind = if is_leap_year(year) { "leap" } else { "common" };
    println!("{year:>5}: {kind}");
}
```

```text
 -748: leap
    1: common
 1500: leap
 1582: common
 1700: common
 1900: common
 2000: leap
 2024: leap
```

`1500` is a leap year (Julian: divisible by 4) while `1700` and `1900` are not (Gregorian). Year
1582 itself uses the Julian rule and is not divisible by 4.

## 2.2 Looping over ranges

```rust
for year in 1896..=1912 {
    if is_leap_year(year) {
        print!("{year} ");
    }
}
```

```text
1896 1904 1908 1912
```

`a..b` excludes `b`, `a..=b` includes it. 1900 is missing: it is a century not divisible by 400.

The loops of Rust:

| Loop | Use |
|---|---|
| `for x in iterator` | iterate over anything iterable: ranges, arrays, `Vec`, ... |
| `while cond { }` | repeat while a condition holds |
| `loop { }` | repeat forever until `break`; `break value` makes the loop an expression |

## 2.3 Month lengths: `match` and tables

### With `match`

```rust
fn days_in_month_match(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if is_leap_year(year) { 29 } else { 28 }),
        _ => None,
    }
}
```

- `match` compares a value against *patterns* in order and runs the first arm that fits.
- `|` combines alternatives; `_` matches anything. `match` must be **exhaustive**: remove the
  `_` arm and the compiler refuses to build, because `u8` has values other than 1-12.
- Every arm yields the same type, so `match` is an expression.

### `Option`

The function returns `Option<u8>` because "month 13" has no length. `Option<T>` is a plain enum:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

There is no `null` in Rust. If a value might be absent, the type says so and the compiler forces
you to handle both cases.

```rust
println!("days_in_month(2024, 13) = {:?}", days_in_month(2024, 13));   // None
if let Some(n) = month_number("Smarch") {
    println!("unexpected {n}");
} else {
    println!("\"Smarch\" is not a month");
}
```

`if let PATTERN = EXPR { ... } else { ... }` is a short `match` for one interesting pattern.
Common `Option` helpers: `unwrap()` (panics on `None`), `expect("msg")`, `unwrap_or(default)`,
`map(f)`, `ok_or(err)` (to a `Result`, see chapter 4), `is_some()`/`is_none()`.

### With a table

```rust
const MONTH_LENGTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn days_in_month(year: i32, month: u8) -> Option<u8> {
    if !(1..=12).contains(&month) {
        return None;
    }
    if month == 2 && is_leap_year(year) {
        return Some(29);
    }
    Some(MONTH_LENGTHS[usize::from(month) - 1])
}
```

This is the version in the real project. Details:

- `return` exits early; use it for guard clauses.
- `(1..=12).contains(&month)` asks whether a value is in a range. A range is a value too.
- Array indexes must be `usize`, hence `usize::from(month)`. Indexing is bounds-checked: an
  out-of-range index panics, which is why we guard first.
- `MONTH_LENGTHS` is a `const`: it is inlined where used and costs nothing at run time.

The two versions must agree, so the tests compare them for every month, including 0 and 13:

```rust
#[test]
fn match_and_table_versions_agree() {
    for year in [1900, 2000, 2023, 2024] {
        for month in 0..=13 {
            assert_eq!(days_in_month(year, month), days_in_month_match(year, month));
        }
    }
}
```

`assert_eq!(left, right)` prints both values on failure; this works because `Option<u8>`
implements `Debug` and `PartialEq`.

## 2.4 Month names and the `?` operator on `Option`

```rust
const MONTH_NAMES: [&str; 12] = ["January", "February", /* ... */ "December"];

fn month_name(month: u8) -> Option<&'static str> {
    MONTH_NAMES.get(usize::from(month).checked_sub(1)?).copied()
}

fn month_number(name: &str) -> Option<u8> {
    MONTH_NAMES
        .iter()
        .position(|n| *n == name)
        .map(|i| (i + 1) as u8)
}
```

- `&str` is a borrowed string slice. A string literal has type `&'static str`: it lives for the
  whole program. (Chapter 5 explains the `'static`.)
- `usize::from(month).checked_sub(1)` is `None` for month 0, instead of underflowing. The `?`
  after it means: if this is `None`, return `None` from the whole function right now; otherwise
  unwrap the value. `?` works on `Option` in functions that return `Option`.
- `slice.get(i)` is the non-panicking index: it returns `Option<&T>`. `.copied()` turns an
  `Option<&&str>` into an `Option<&str>`.
- `.iter().position(|n| *n == name)` finds the index of the first match. `|n| ...` is a closure
  (chapter 7). The result is `Option<usize>`; `.map(...)` converts the inner value only if it is
  `Some`.

## 2.5 The gap of 1582 with `matches!`

The Gregorian reform went from 1582-10-04 straight to 1582-10-15.

```rust
fn is_skipped_by_reform(year: i32, month: u8, day: u8) -> bool {
    matches!((year, month, day), (1582, 10, 5..=14))
}
```

`matches!(expr, pattern)` returns `true` if the value fits the pattern. Here the value is a tuple
and the pattern combines literals with a range. It is equivalent to:

```rust
match (year, month, day) {
    (1582, 10, 5..=14) => true,
    _ => false,
}
```

```text
1582-10-04: false
1582-10-10: true
1582-10-15: false
```

## 2.6 `while` and `loop`

```rust
/// 1-based day number within the year, or `None` for an invalid date.
fn day_of_year(year: i32, month: u8, day: u8) -> Option<u16> {
    let max_day = days_in_month(year, month)?;
    if day == 0 || day > max_day {
        return None;
    }
    let mut total = u16::from(day);
    let mut m = 1;
    while m < month {
        total += u16::from(days_in_month(year, m)?);
        m += 1;
    }
    Some(total)
}
```

```text
2024-03-01: Some(61)
2023-03-01: Some(60)
2023-02-29: None
```

The date is validated first, then the lengths of the preceding months are added up. `let mut`
makes `total` and `m` changeable.

`loop` is for "repeat until something is found". Its `break` can carry a value:

```rust
fn first_leap_year_after(start: i32) -> i32 {
    let mut year = start;
    loop {
        year += 1;
        if is_leap_year(year) {
            break year; // `loop` is an expression: `break` carries the value
        }
    }
}
```

`first_leap_year_after(1899)` is `1904`, because 1900 is a common year.

## 2.7 Expressions versus statements

Rust is expression-oriented. Almost everything produces a value: `if`, `match`, blocks `{ ... }`,
`loop`. A trailing semicolon turns an expression into a statement whose value is `()`. The most
common beginner mistake is adding one too many:

```rust
fn is_leap_year(year: i32) -> bool {
    year % 4 == 0;      // error[E0308]: mismatched types: expected `bool`, found `()`
}
```

## 2.8 Run it

```sh
cargo run -p ch2
cargo test -p ch2
```

The output of the month table:

```text
 1 January    31 31
 2 February   28 29
 3 March      31 31
 ...
12 December   31 31
```

**In the real project:** [`calendar.rs`](../../../ptolemy/crates/ptolemy-time/src/calendar.rs)
contains exactly `is_leap_year`, `days_in_month`, `month_name` and `month_number`, and
`HistoricDateTime::new` (see chapter 4) uses them to validate dates. The same rules appear in the
Python original as `isleapyear` in `basicdefs`.

## Exercises

1. Add `fn days_in_year(year: i32) -> u16` and test it for 1500, 1700, 2000 and 2023. (Chapter 8
   shows a solution.)
2. Write `fn weekday(jd: f64) -> &'static str` using `((jd + 1.5) as i64).rem_euclid(7)` and a
   `match` on the result 0..=6 (0 is Sunday). Check that 2000-01-01.5 (JD 2451545.0) was a
   Saturday. Which arm do you need for the "impossible" values, and why is it still required?
3. Change `days_in_month` so that it returns `Result<u8, String>` with the message
   `"month must be between 1 and 12, got N"`. (Chapter 4 does this properly.)
4. Remove the `_ => None` arm of `days_in_month_match` and read the compiler's error message.
5. Write `fn last_day_of_february(year: i32) -> u8` using only `if` as an expression, with no
   `return` and no `let`.

## Summary

- Functions return their last expression; `if`, `match` and `loop` are expressions.
- `match` is exhaustive and supports alternatives, ranges and tuples; `matches!` is its boolean
  shorthand.
- `Option<T>` models "may be absent"; `?`, `if let`, `map`, `ok_or` and `unwrap_or` are the tools.
- `for` iterates, `while` tests, `loop` repeats until `break` (optionally with a value).
- Constant tables are arrays; `.get()` is the non-panicking index.

Previous: [Chapter 1](../ch1/README.md) | Next: [Chapter 3: Custom types](../ch3/README.md)
