# Chapter 3: Custom types

Programs get readable when the data has names. Python's original `astronomy` package passes
coordinates as loose floats and selects units with strings such as `'HOUR'` and `'DEG'`. In Rust
you describe the data with `struct` and `enum` types, and the compiler checks every use.

**You will learn**

- `struct`: named fields, tuple structs (newtypes), unit structs
- `enum`: plain variants, variants with data, discriminants
- `impl` blocks: associated functions, methods and `Self`
- `#[derive]`: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`
- struct update syntax and destructuring
- privacy: private fields as an encapsulation tool
- `const fn` and constant tables of structs

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch3`.

## 3.1 Structs with named fields

A position on the sky, in degrees:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Equatorial {
    ra_deg: f64,
    dec_deg: f64,
}

impl Equatorial {
    fn new(ra_deg: f64, dec_deg: f64) -> Self {
        Self { ra_deg, dec_deg }
    }

    fn from_hours(ra_hours: f64, dec_deg: f64) -> Self {
        Self::new(ra_hours * 15.0, dec_deg)
    }

    fn ra_hours(&self) -> f64 {
        self.ra_deg / 15.0
    }
}
```

- `impl Type { ... }` attaches functions to a type. `Self` is an alias for the type.
- A function without `self` is an **associated function**, called as `Equatorial::new(..)`. By
  convention `new` is the primary constructor, and `from_*` are alternative ones.
- A function with `self`, `&self` or `&mut self` is a **method**, called as `value.method()`.
  `&self` borrows the value read-only (chapter 5).
- *Field init shorthand*: `Self { ra_deg, dec_deg }` means `Self { ra_deg: ra_deg, dec_deg: dec_deg }`.
- Names encode the unit (`ra_deg`, `ra_hours`). Better still, make the unit part of the type, as in
  section 3.4.

```rust
let vega = Equatorial::from_hours(18.615_607_22, 38.782_993_11);
println!("{vega:?}");
println!("{vega:#?}"); // pretty debug
println!("Vega RA = {:.6} h", vega.ra_hours());
```

```text
Equatorial { ra_deg: 279.2341083, dec_deg: 38.78299311 }
Equatorial {
    ra_deg: 279.2341083,
    dec_deg: 38.78299311,
}
Vega RA = 18.615607 h
```

### `derive`

`#[derive(...)]` asks the compiler to generate trait implementations (chapter 6 explains traits):

| Derive | Gives you |
|---|---|
| `Debug` | `{:?}` and `{:#?}` formatting for debugging |
| `Clone` | `.clone()`, an explicit duplicate |
| `Copy` | implicit duplicate on assignment and passing (only for plain data) |
| `PartialEq` / `Eq` | `==` and `!=` (`Eq` additionally promises `a == a`, so it is not available for `f64`) |
| `PartialOrd` / `Ord` | `<`, `>`, `.cmp()`, sorting |
| `Hash` | usable as a `HashMap` key or in a `HashSet` |
| `Default` | `Type::default()` with every field at its default |

`Equatorial` derives `PartialEq` but not `Eq`, because `f64` has `NaN != NaN`. It derives `Copy`
because it is just two numbers: passing it by value is as cheap as passing a reference.

```rust
let copy = vega;                      // a copy; `vega` is still usable
println!("copies are equal: {}", copy == vega);
```

### Struct update syntax and `Default`

```rust
let shifted = Equatorial { dec_deg: -26.43, ..vega };   // other fields from `vega`

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Ecliptic { lon_deg: f64, lat_deg: f64 }

let origin = Ecliptic::default();                        // all zeros
let pollux = Ecliptic { lon_deg: 113.2, ..Default::default() };
```

```text
Equatorial { ra_deg: 279.2341083, dec_deg: -26.43 }
Ecliptic { lon_deg: 0.0, lat_deg: 0.0 }
Ecliptic { lon_deg: 113.2, lat_deg: 0.0 }
```

## 3.2 Tuple structs and unit structs

```rust
struct Degrees(f64);          // tuple struct: fields are .0, .1, ...
struct J2000;                 // unit struct: no data at all

let Degrees(inner) = Degrees(45.0);   // destructuring
```

A tuple struct with one field is the **newtype pattern**: it creates a distinct type with zero
run-time cost. Giving `Degrees(f64)` and `Radians(f64)` different types makes it a compile error
to pass radians to a function that expects degrees. The real project keeps plain `f64` degrees
everywhere and documents it; a newtype is a good refactoring exercise (see below).

## 3.3 Enums

An `enum` is a type with a fixed set of alternatives ("sum type"). The Python original compared
strings; a typo such as `'HOURS'` failed only at run time. The enum makes invalid values
unrepresentable:

```rust
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
```

```text
Hours: 6.0 -> 90 deg
Degrees: 6.0 -> 6 deg
```

Add a variant `Radians` and every `match self` without a `Radians` arm stops compiling, so the
compiler lists exactly the places you have to update.

### Enums that carry data

Each variant may hold its own data, with unnamed or named fields:

```rust
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
```

`match` destructures the variant and binds its fields to local names. This chapter's
`RightAscension` is an illustration, not part of the real crate; the real code separates the unit
(`RaUnit`) from the number, to keep the batch functions simple (chapter 11). `Option<T>` and
`Result<T, E>` are exactly such enums: `Some(T)`/`None` and `Ok(T)`/`Err(E)`.

### Enums with discriminants: the Egyptian months

The Egyptian year has 13 "months": twelve of 30 days and the five *epagomenal* days at the end.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EgyptianMonth {
    Thoth, Phaophi, Athyr, Choiak, Tybi, Mechir, Phamenoth,
    Pharmouthi, Pachon, Payni, Epiphi, Mesore, Epagomenal,
}

impl EgyptianMonth {
    /// All variants in calendar order; enums cannot be iterated directly.
    const ALL: [EgyptianMonth; 13] = [ /* all 13 variants, in order */ ];

    fn number(self) -> u8 {
        self as u8 + 1          // discriminants are 0, 1, 2, ...
    }

    fn length(self) -> u8 {
        if self == EgyptianMonth::Epagomenal { 5 } else { 30 }
    }
}
```

```rust
let m = EgyptianMonth::Epagomenal;
println!("{m:?}: number {}, length {}", m.number(), m.length());
let days: u32 = EgyptianMonth::ALL.iter().map(|m| u32::from(m.length())).sum();
println!("{} months, {days} days in a year", EgyptianMonth::ALL.len());
```

```text
Epagomenal: number 13, length 5
13 months, 365 days in a year
```

- A fieldless enum can be cast with `as u8`; the first variant is 0 unless you write
  `Thoth = 1`.
- `ALL` is an associated `const`. The `iter().map(..).sum()` chain is covered in chapter 7; the
  check `12 * 30 + 5 == 365` is a good sanity test of the data.

## 3.4 Privacy and invariants

By default, fields are private to the module that defines the type. That is how a type guarantees
that its values are valid:

```rust
const SECS_PER_DAY: i64 = 86_400;
const SECS_PER_YEAR: i64 = 365 * SECS_PER_DAY;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign { Plus, Minus }

/// The field is private (no `pub`), so every value goes through our constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Duration {
    total_secs: i64,
}

impl Duration {
    const fn from_total_secs(total_secs: i64) -> Self { Self { total_secs } }

    fn sign(&self) -> Sign {
        if self.total_secs < 0 { Sign::Minus } else { Sign::Plus }
    }

    fn magnitude(&self) -> u64 { self.total_secs.unsigned_abs() }
    fn years(&self) -> u64 { self.magnitude() / SECS_PER_YEAR as u64 }
    fn days(&self) -> u32 {
        (self.magnitude() % SECS_PER_YEAR as u64 / SECS_PER_DAY as u64) as u32
    }
}
```

```rust
let d = Duration::from_total_secs(-(2 * SECS_PER_YEAR + 3 * SECS_PER_DAY));
println!("{d:?}: {:?} {} years {} days", d.sign(), d.years(), d.days());
```

```text
Duration { total_secs: -63331200 }: Minus 2 years 3 days
```

Design notes, taken from the real [`duration.rs`](../../../ptolemy/crates/ptolemy-time/src/duration.rs):

- The Python class stored sign, years, days, hours, minutes and seconds separately, so many states
  could contradict each other (for example a negative zero). Here **one** integer is the source of
  truth and the components are *derived* on demand. Invalid states cannot exist.
- `const fn` can be evaluated at compile time; it also allows `const ZERO: Duration = ...`.
- `unsigned_abs()` avoids the overflow of `i64::MIN.abs()`.

## 3.5 A constant table of structs

The Egyptian calendar needs the list of rulers, with their regnal years, because the *Almagest*
counts "year 17 of Hadrian":

```rust
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
```

The short helper `r(..)` keeps the 55-row table in the real
[`egyptian.rs`](../../../ptolemy/crates/ptolemy-time/src/egyptian.rs) readable. The type of the array,
`[Reign; 4]`, includes its length; the real table is `[Reign; 55]`.

## Exercises

1. Add a `Radians(f64)` variant to `RightAscension` and let the compiler tell you what to fix.
2. Turn `Equatorial` into one that uses newtypes: `ra: Degrees`, `dec: Degrees`. How does the
   code of `from_hours` change? What do you gain and what does it cost?
3. Add `fn is_epagomenal(self) -> bool` to `EgyptianMonth`, written with `matches!`.
4. Add `fn is_negative(&self) -> bool` to `Duration`. Why is it unnecessary to store the sign
   separately?
5. Try to build a `Duration` with a struct literal outside the `impl` block, but from another
   module (wrap everything in `mod time { ... }` and mark only `Duration` as `pub`). Read the
   error message. This is the foundation of chapter 8.
6. Add `#[derive(PartialOrd)]` to `Equatorial`. What does the derived ordering compare first?

## Summary

- `struct` groups named fields; tuple structs make newtypes; unit structs carry no data.
- `enum` describes alternatives, possibly with data; `match` must cover all of them.
- `impl` adds associated functions (`Type::new`) and methods (`value.method()`).
- `#[derive]` generates common trait implementations; plain-data types can be `Copy`.
- Private fields plus constructors keep values valid.

Previous: [Chapter 2](../ch2/README.md) | Next: [Chapter 4: Error handling](../ch4/README.md)
