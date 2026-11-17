# Chapter 5: Ownership, borrowing and lifetimes

Ownership is the idea that sets Rust apart: memory is managed without a garbage collector and
without manual `free`, and the compiler proves that no reference outlives its data. The rules are
few, and the project's code exercises all of them: values that are copied (`HistoricDateTime`),
values that are moved (a list of stars), borrowed text (a catalogue parser) and `'static` data (the
list of Egyptian kings).

**You will learn**

- the three ownership rules, moves, `Clone` and `Copy`
- passing by value, by shared reference `&T` and by exclusive reference `&mut T`
- the borrowing rules, and non-lexical lifetimes
- lifetime annotations (`'a`) and elision, `'static`
- structs that hold references
- what the compiler's errors look like and how to fix them

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch5`.

## 5.1 The rules

1. Every value has exactly one **owner**.
2. When the owner goes out of scope, the value is **dropped** (freed).
3. Ownership can be **moved** to another owner, or **borrowed** with references.

At any time you can have either any number of shared references (`&T`) or exactly one exclusive
reference (`&mut T`), never both. References must never outlive the value they point to.

These rules are checked at compile time, so they cost nothing at run time.

## 5.2 Move, `Clone`, `Copy`

```rust
let a = String::from("Vega");
let b = a;                // `a` is moved into `b`
println!("{b}");
// println!("{a}");       // error[E0382]: borrow of moved value: `a`

let c = b.clone();        // explicit deep copy
println!("{b} {c}");
```

A `String` owns a heap buffer. Assignment moves the ownership; copying the pointer would lead to a
double free, so the old name becomes unusable. If you really want two strings, say so with
`.clone()`.

Small plain-data types are different: they implement `Copy`, and assignment duplicates the bits:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Equatorial { ra_deg: f64, dec_deg: f64 }

let p = Equatorial { ra_deg: 279.23, dec_deg: 38.78 };
let q = p;                // copy
println!("{p:?} {q:?}");  // both usable
```

| Type | Copy? | Why |
|---|---|---|
| `i32`, `f64`, `bool`, `char` | yes | plain bits |
| `&T` (shared reference) | yes | a pointer |
| tuples and arrays of `Copy` types | yes | |
| your struct, if every field is `Copy` and you `derive(Copy)` | opt in | `HistoricDateTime`, `Duration`, `Equatorial` |
| `String`, `Vec<T>`, `Box<T>` | no | own heap memory |
| `&mut T` | no | exclusivity |

The real `HistoricDateTime` is `Copy` (six small integers), so you can pass dates around without
thinking about ownership.

## 5.3 Three ways to take a parameter

```rust
#[derive(Debug, Clone, PartialEq)]
struct Star {
    name: String,            // owns its text
    position: Equatorial,
}

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
```

| Signature | Meaning | Call |
|---|---|---|
| `fn f(x: T)` | consumes `x` | `f(x)` |
| `fn f(x: &T)` | reads `x` | `f(&x)` |
| `fn f(x: &mut T)` | modifies `x` | `f(&mut x)` |

Prefer `&[T]` over `&Vec<T>` and `&str` over `&String` for parameters: callers with an array, a
`Vec` or a slice all work. A `Vec<Star>` coerces to `&[Star]` automatically.

```rust
let mut stars = vec![ /* Vega, Antares */ ];
println!("mean RA = {:.3}", mean_ra(&stars));
shift_ra(&mut stars, 90.0);
println!("mean RA after shift = {:.3}", mean_ra(&stars));
```

```text
mean RA = 263.290
mean RA after shift = 173.290
```

(Check: `279.23 + 90` wraps to `9.23`, `247.35 + 90 = 337.35`, mean `173.29`.)

### The borrowing rules in action

```rust
let first = &stars[0];                  // shared borrow starts
println!("first = {}", first.name);     // last use of `first`: the borrow ends here
shift_ra(&mut stars, 0.0);              // so an exclusive borrow is fine now
```

Since the 2018 edition borrows end at their *last use*, not at the end of the block (*non-lexical
lifetimes*). Had `first` been used after the `shift_ra` call, the compiler would say:

```text
error[E0502]: cannot borrow `stars` as mutable because it is also borrowed as immutable
```

That rule prevents a whole class of bugs: while somebody holds `&stars[0]`, a push into `stars`
could reallocate the buffer and leave that reference dangling.

Finally, the vector is consumed:

```rust
let names = into_names(stars);          // `stars` is moved
println!("{names:?}");                  // ["Vega", "Antares"]
// println!("{}", stars.len());         // error[E0382]: borrow of moved value: `stars`
```

## 5.4 Lifetimes

A **lifetime** is the span of code in which a reference is valid. Usually the compiler infers it.
You write them only when a function returns a reference and the compiler cannot tell which input it
comes from.

### Elision

```rust
fn parse_line(line: &str) -> Option<(&str, f64, f64)> {
    let mut fields = line.split_whitespace();
    let name = fields.next()?;
    let ra_hours = fields.next()?.parse().ok()?;
    let dec = fields.next()?.parse().ok()?;
    Some((name, ra_hours, dec))
}
```

With one input reference, the output reference gets the same lifetime. It is shorthand for
`fn parse_line<'a>(line: &'a str) -> Option<(&'a str, f64, f64)>`. The returned name is a view into
`line`: nothing is copied. `.parse().ok()?` turns a failed number parse into `None` and returns
early.

### Explicit lifetimes

```rust
/// Two inputs, so elision cannot guess: the result lives as long as both do.
fn longest_reign<'a>(a: &'a Reign, b: &'a Reign) -> &'a Reign {
    if b.regnal_years > a.regnal_years { b } else { a }
}
```

`'a` names a region. The signature promises: "the result is valid for as long as *both* inputs
are". A lifetime annotation does not lengthen anything; it only describes relationships so the
compiler can check callers.

### `'static`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Reign {
    name: &'static str,
    regnal_years: u32,
}

const REIGNS: [Reign; 8] = [ /* ... */ ];

/// `'static` data lives for the whole program, so the result does not depend on `name`.
fn find_reign(name: &str) -> Option<&'static Reign> {
    REIGNS.iter().find(|r| r.name == name)
}
```

String literals have type `&'static str`: they are baked into the program binary and live forever.
A `&'static Reign` can be returned from `find_reign` even though `name` is a short-lived parameter,
because the result points into the table, not into `name`.

In the real [`egyptian.rs`](../../../ptolemy/crates/ptolemy-time/src/egyptian.rs) the `Egyptian`
struct stores `reign: &'static str`. `Egyptian::new` takes a `&str` (any lifetime), looks it up in
`REIGNS`, and stores the *table's* name. A validated date therefore needs no allocation and never
dangles.

### Structs that borrow

```rust
struct Catalogue<'a> {
    entries: Vec<(&'a str, f64, f64)>,
}

impl<'a> Catalogue<'a> {
    fn parse(text: &'a str) -> Self {
        Self { entries: text.lines().filter_map(parse_line).collect() }
    }

    fn names(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.entries.iter().map(|(name, _, _)| *name)
    }
}
```

`Catalogue<'a>` cannot outlive the text it was parsed from; the compiler enforces it.

```rust
let text = "Vega 18.615 38.783\nAntares 16.490 -26.432\nbroken line\nSpica 13.420 -11.161";
let catalogue = Catalogue::parse(text);
println!("{} usable lines: {:?}", catalogue.entries.len(), catalogue.names().collect::<Vec<_>>());
```

```text
3 usable lines: ["Vega", "Antares", "Spica"]
```

The "broken line" is dropped by `filter_map`, because `parse_line` returns `None` for it.
`names()` returns an iterator borrowing from the catalogue; `+ '_` says so (`impl Iterator`
is explained in chapter 7).

```rust
let nabunasir = find_reign("Nabunasir").expect("known reign");
let mardukempad = find_reign("Mardukempad").expect("known reign");
println!("longest: {}", longest_reign(nabunasir, mardukempad).name);   // Nabunasir (14 > 12)
println!("unknown: {:?}", find_reign("Nobody"));                       // None
```

### A reference may not outlive its owner

```rust
let dangling: &String;
{
    let s = String::from("x");
    dangling = &s;
}   // `s` is dropped here
// println!("{dangling}");   // error[E0597]: `s` does not live long enough
```

In C this would be a use-after-free. In Rust it does not compile.

## 5.5 When ownership gets in the way

| Problem | Typical answer |
|---|---|
| need two owners | `.clone()`, or share with `Rc<T>` / `Arc<T>` (not needed in this project) |
| need to mutate through a shared reference | `Cell`/`RefCell` (single thread), `Mutex` (threads) |
| struct needs a long-lived reference | store an owned value (`String`) instead, unless allocation matters |
| fighting the borrow checker | restructure: split a struct, shorten a borrow, return an index instead of a reference |

The project avoids all of these: its types are small `Copy` values, results are returned by value,
and the only reference-holding data are `&'static str` names in constant tables.

## Exercises

1. Uncomment the moved-value line (`println!("{a}")`) and read the full error. Fix it in three ways:
   `clone`, borrowing (`let b = &a;`), and by removing the use.
2. Make `shift_ra` take `&mut Vec<Star>` and then call it with a slice. What does the compiler say?
3. Write `fn brightest<'a>(stars: &'a [Star], magnitudes: &[f64]) -> Option<&'a Star>`. Why is
   `'a` only needed on the first parameter and the result?
4. Write a function that takes `&mut Vec<Star>` and sorts it by `ra_deg` with
   `sort_by(|a, b| a.position.ra_deg.total_cmp(&b.position.ra_deg))`.
5. Remove `Copy` from `Equatorial` in this chapter's code. Which lines no longer compile? How would
   you fix each?
6. Make `Catalogue` own its names (`Vec<(String, f64, f64)>`). Which lifetimes disappear? What is
   the price?

## Summary

- One owner per value; assignment moves unless the type is `Copy`.
- Borrow with `&T` (many readers) or `&mut T` (one writer); borrows end at their last use.
- Lifetimes describe how long references stay valid; elision hides most of them, `'static` means
  "for the whole program".
- Prefer small `Copy` types and borrowed parameters; reach for `clone` deliberately.

Previous: [Chapter 4](../ch4/README.md) | Next: [Chapter 6: Traits](../ch6/README.md)
