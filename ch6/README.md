# Chapter 6: Traits

A *trait* describes behaviour that types can share, like an interface in other languages. Rust's
standard library expresses almost everything through traits: printing (`Display`), comparing
(`PartialOrd`), arithmetic (`Add`), parsing (`FromStr`), conversion (`From`). By implementing
them for the project's own types, `Duration` and `EgyptianMonth` feel like built-in types.

**You will learn**

- implementing `Display` and using `to_string()`
- operator overloading: `Add`, `Sub`, `Neg`, `AddAssign`, `SubAssign`
- ordering and equality: `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and derived ordering
- `From`/`Into` and `FromStr`/`parse`
- defining your own trait, default methods, generic functions with trait bounds
- static versus dynamic dispatch (`impl Trait`, `dyn Trait`, `Box<dyn Trait>`)
- the orphan rule

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch6`.

## 6.1 The idea

```rust
trait Summary {
    fn summarize(&self) -> String;
}

impl Summary for Duration {
    fn summarize(&self) -> String { /* ... */ }
}
```

`impl Trait for Type` provides the required methods. If the trait is in scope, the methods can be
called with the usual dot syntax. Many traits can be implemented automatically with
`#[derive(...)]`, as seen in chapter 3.

**Orphan rule:** you may implement a trait for a type only if the trait or the type is defined in
your crate. You can implement `Display` for your `Duration`, and your own trait for `f64`, but not
`Display` for `f64`. This keeps independent crates from conflicting.

## 6.2 `Display`

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Sign { Plus, Minus }

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Sign::Plus => "+",
            Sign::Minus => "-",
        })
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} years {:02} days {:02} hours {:02} minutes {:02} seconds",
            self.sign(), self.years(), self.days(), self.hours(), self.minutes(), self.secs()
        )
    }
}
```

`write!` is `println!` for any destination implementing `fmt::Write`; here the destination is the
formatter. Implementing `Display` gives you `{}` formatting and, for free, `.to_string()`.

`Display` is for end users, `Debug` for programmers. `Display` is never derived because there is
no single right text. The format above is the one of the Python original (`str(duration)`),
checked in the real test `display_matches_original_format`.

```rust
let a = d(Sign::Plus, 1, 364, 23, 59, 59);
println!("a       = {a}");
```

```text
a       = + 1 years 364 days 23 hours 59 minutes 59 seconds
```

## 6.3 Operator overloading

Operators are traits in `std::ops`. `a + b` is sugar for `Add::add(a, b)`.

```rust
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

impl Neg for Duration {
    type Output = Duration;
    fn neg(self) -> Duration { Duration::from_total_secs(-self.total_secs) }
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
    fn add_assign(&mut self, rhs: Duration) { *self = *self + rhs; }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) { *self = *self - rhs; }
}
```

- `type Output` is an *associated type*: the result type of the operation.
- The operators take `self` **by value**. For `Copy` types like `Duration` that is free, and the
  operands are not modified: `a - b` does not change `b`. (The Python `__sub__` mutated `other`,
  one of the bugs fixed in the port.)
- `AddAssign` (`+=`) takes `&mut self`; implementing it via `+` keeps the logic in one place.

```rust
let a = d(Sign::Plus, 1, 364, 23, 59, 59);
let b = d(Sign::Plus, 0, 0, 0, 0, 1);
println!("a + b   = {}", a + b);
println!("b - a   = {}", b - a);
println!("-a      = {}", -a);

let mut acc = Duration::default();
acc += a;
acc -= d(Sign::Plus, 3, 0, 0, 0, 0);
println!("acc     = {acc}");
```

```text
a + b   = + 2 years 00 days 00 hours 00 minutes 00 seconds
b - a   = - 1 years 364 days 23 hours 59 minutes 58 seconds
-a      = - 1 years 364 days 23 hours 59 minutes 59 seconds
acc     = - 1 years 00 days 00 hours 00 minutes 01 seconds
```

Adding one second to 1 year 364 days 23:59:59 carries all the way to 2 years: the components are
derived from a single integer, so no carry code exists to get wrong.

## 6.4 Equality and ordering

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Duration { total_secs: i64 }
```

| Trait | Operators / methods | Notes |
|---|---|---|
| `PartialEq` | `==`, `!=` | partial: `f64::NAN != f64::NAN` |
| `Eq` | (marker) | full equivalence; required for `HashMap` keys |
| `PartialOrd` | `<`, `<=`, `>`, `>=`, `partial_cmp` | partial: `NaN` has no order |
| `Ord` | `cmp`, `max`, `min`, `sort()` | total order |
| `Hash` | | must agree with `Eq`: equal values hash equally |

Derived ordering compares **fields in declaration order**, then lexicographically. With the single
field `total_secs: i64`, `Duration`'s order is the order of signed totals, which is the order
people expect: `-5 years < -1 year < +1 year`. The Python version compared magnitudes and ignored
the sign.

The real `HistoricDateTime` relies on the same rule: its fields are declared
`year, month, day, hour, minute, sec` and it derives `Ord`, so comparing two dates compares them
chronologically. Reordering the fields would silently break it; that is why the source carries the
comment `// Field order defines chronological ordering.`

```rust
println!("a < b?    {}", a < b);       // false
println!("max       {}", a.max(b));

let mut sorted = [b, a, -a, Duration::default()];
sorted.sort();                          // needs `Ord`
```

```text
sorted  : - 1 years 364 days 23 hours 59 minutes 59 seconds
sorted  : + 0 years 00 days 00 hours 00 minutes 00 seconds
sorted  : + 0 years 00 days 00 hours 00 minutes 01 seconds
sorted  : + 1 years 364 days 23 hours 59 minutes 59 seconds
```

`f64` is only `PartialOrd`, so `[f64]::sort()` does not compile. Use `sort_by(|a, b|
a.total_cmp(b))` (chapter 7).

## 6.5 `From` and `Into`

```rust
impl From<i64> for Duration {
    fn from(total_secs: i64) -> Self { Duration::from_total_secs(total_secs) }
}

let from_int: Duration = 3_600.into();   // `Into` is implemented for you
```

```text
1 hour  = + 0 years 00 days 01 hours 00 minutes 00 seconds
```

Implement `From<A> for B`, and `A: Into<B>` comes automatically. `From` is also what makes the `?`
operator convert errors (chapter 4). Use `From` only for conversions that cannot fail; for failing
ones use `TryFrom` or `FromStr`.

## 6.6 `FromStr` and `parse`

The Python function returned `-1` for an unknown month name, which then silently corrupted later
arithmetic. In Rust the month implements `FromStr`, and parsing returns a `Result`:

```rust
use std::str::FromStr;

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
```

`str::parse::<T>()` works for every `T: FromStr`, including `i32`, `f64` and now our enum:

```rust
let month: EgyptianMonth = "Payni".parse().unwrap();
println!("{month} is month {}", month.number());
println!("{:?}", "Smarch".parse::<EgyptianMonth>());
```

```text
Payni is month 10
Err(ParseMonthError("Smarch"))
unknown Egyptian month "Smarch"
```

`::<EgyptianMonth>` is the *turbofish*, used when inference has no other hint.
`ok_or_else` is like `ok_or`, but builds the error only if needed.

## 6.7 Your own trait

```rust
/// A point on the celestial sphere in some coordinate system.
trait SkyPosition {
    fn system(&self) -> &'static str;
    fn longitude_like(&self) -> f64;
    fn latitude_like(&self) -> f64;

    /// A default method; implementors may override it.
    fn describe(&self) -> String {
        format!("{}: {:.3} / {:.3}", self.system(), self.longitude_like(), self.latitude_like())
    }
}

impl SkyPosition for Equatorial {
    fn system(&self) -> &'static str { "equatorial" }
    fn longitude_like(&self) -> f64 { self.ra_deg }
    fn latitude_like(&self) -> f64 { self.dec_deg }
}

impl SkyPosition for Ecliptic {
    fn system(&self) -> &'static str { "ecliptic" }
    fn longitude_like(&self) -> f64 { self.lon_deg }
    fn latitude_like(&self) -> f64 { self.lat_deg }
    fn describe(&self) -> String {                      // override
        format!("ecliptic (lambda, beta) = ({:.3}, {:.3})", self.lon_deg, self.lat_deg)
    }
}
```

### Static dispatch: generics with bounds

```rust
fn print_generic<P: SkyPosition>(p: &P) {
    println!("generic : {}", p.describe());
}
```

The compiler generates a specialised copy of `print_generic` for each concrete type (*monomorphisation*):
the call is direct and can be inlined. `fn f(p: &impl SkyPosition)` is shorthand for the same.

### Dynamic dispatch: trait objects

```rust
fn print_all(points: &[Box<dyn SkyPosition>]) {
    for p in points {
        println!("dyn     : {}", p.describe());
    }
}

print_all(&[Box::new(eq), Box::new(ecl)]);   // a Vec of *different* types
```

```text
generic : equatorial: 116.329 / 28.026
generic : ecliptic (lambda, beta) = (113.216, 6.684)
dyn     : equatorial: 116.329 / 28.026
dyn     : ecliptic (lambda, beta) = (113.216, 6.684)
```

`dyn Trait` is looked up through a vtable at run time, which allows heterogeneous collections at a
small cost. A trait must be *object safe* (no generic methods, no `Self` return types) to be used as
`dyn`. Rule of thumb: generics first; `dyn` when you need a mixed collection or want to reduce code
size.

Note that the real project does **not** define such a trait. Its `Equatorial` and `Ecliptic` are
plain structs, and conversion is done by functions. Introducing a trait would be a design choice
with a cost (an extra abstraction to learn), and this chapter does it only to show the mechanism.

## 6.8 Trait cheat sheet for this project

| Type | Traits it implements |
|---|---|
| `Sign` | `Debug Clone Copy PartialEq Eq Hash Display` |
| `Duration` | the above plus `PartialOrd Ord Neg Add Sub AddAssign SubAssign` |
| `HistoricDateTime` | `Debug Clone Copy PartialEq Eq Hash PartialOrd Ord Display` |
| `EgyptianMonth` | `Debug Clone Copy PartialEq Eq Hash Display FromStr` |
| `Equatorial`, `Ecliptic` | `Debug Clone Copy PartialEq` |
| all error enums | `Debug Clone PartialEq Eq Error` (via `thiserror`) |

**In the real project:** [`duration.rs`](../../../ptolemy/crates/ptolemy-time/src/duration.rs) has the
operator implementations, and [`egyptian.rs`](../../../ptolemy/crates/ptolemy-time/src/egyptian.rs)
the `FromStr` for `EgyptianMonth` (with a `thiserror` error instead of the hand-written one
above).

## Exercises

1. Implement `std::iter::Sum` for `Duration` so that `durations.iter().copied().sum::<Duration>()`
   works.
2. Implement `Mul<i64> for Duration` and `Div<i64> for Duration`. What should dividing by zero do?
3. Implement `Display` for `Equatorial` in sexagesimal form: `18h 36m 56.2s +38° 46' 58.8"`.
4. Implement `TryFrom<u8> for EgyptianMonth` from the month number 1..=13.
5. Add a second default method `fn is_northern(&self) -> bool` to `SkyPosition` (latitude-like
   `>= 0`). Does `Equatorial` need to change?
6. Make a trait method generic (`fn map<F: Fn(f64) -> f64>(&self, f: F) -> Self`). What error do you
   get when you then try to build `Box<dyn SkyPosition>`?

## Summary

- Traits define shared behaviour; `impl Trait for Type` provides it; `derive` writes the usual ones.
- Operators, formatting, parsing and conversions are all traits, so your types integrate with the
  standard library.
- Derived ordering follows field order.
- Generics with bounds give static dispatch; `dyn Trait` gives dynamic dispatch.

Previous: [Chapter 5](../ch5/README.md) | Next: [Chapter 7: Generics, iterators, closures](../ch7/README.md)
