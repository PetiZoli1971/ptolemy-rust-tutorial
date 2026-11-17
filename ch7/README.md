# Chapter 7: Generics, iterators and closures

The Python original loops over lists with index variables and `numpy` arrays. Idiomatic Rust
describes the same computation as a pipeline of iterator adaptors, parametrises types and
functions with generics, and passes behaviour around as closures. Despite the abstraction, the
compiler usually turns these pipelines into the same machine code as the hand-written loop.

**You will learn**

- generic structs and functions, `impl<T>`, trait bounds, `where` clauses
- iterator adaptors: `map`, `filter`, `scan`, `find`, `partition`, `max_by_key`, `fold`, `zip`, `rev`, `take`
- consuming adaptors: `collect`, `sum`, `fold`
- returning `impl Iterator`
- closures: capture by reference and by `move`, the `Fn`/`FnMut`/`FnOnce` traits, `Box<dyn Fn>`
- why floats need `total_cmp` and `f64::max`

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch7`.

## 7.1 Generics

A **generic** type has a placeholder for another type. The standard library's `Option<T>`,
`Result<T, E>` and `Vec<T>` are generic. The real project has one too: `Checked<T>`, which
pairs the result of a conversion with a quality indicator.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Checked<T> {
    value: T,
    gradient: f64,
}

impl<T> Checked<T> {
    /// Transform the payload and keep the metadata. `U` is chosen by the closure.
    fn map<U>(self, f: impl FnOnce(T) -> U) -> Checked<U> {
        Checked { value: f(self.value), gradient: self.gradient }
    }
}
```

`Checked<Ecliptic>`, `Checked<Equatorial>` and `Checked<f64>` all use the same code; the compiler
generates one specialised version per type actually used (*monomorphisation*), so generics are
free at run time.

```rust
let c = Checked { value: 18.6156_f64, gradient: 1.2 };
let hours_to_deg = c.map(|h| h * 15.0);
let as_text = hours_to_deg.map(|d| format!("{d:.3} deg"));
println!("{c:?}\n{hours_to_deg:?}\n{as_text:?}");
```

```text
Checked { value: 18.6156, gradient: 1.2 }
Checked { value: 279.23400000000004, gradient: 1.2 }
Checked { value: "279.234 deg", gradient: 1.2 }
```

(`279.23400000000004` is ordinary floating-point rounding: see chapter 9.)

### Trait bounds

`T` can be anything, so inside a generic function you can do nothing with it. A *bound* says what
`T` must support:

```rust
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter().copied();
    let first = iter.next()?;
    Some(iter.fold(first, |best, x| if x > best { x } else { best }))
}
```

`PartialOrd` provides `>`, `Copy` lets us return values out of the slice. For longer bounds a
`where` clause reads better:

```rust
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
```

`T: Into<f64>` is satisfied by `u8`, `i32`, `f32` and `f64` (all lossless conversions), but not by
`i64` or `usize`; try it and read the error.

```rust
println!("largest i32: {:?}", largest(&[3, 9, 4]));
println!("largest f64: {:?}", largest(&[0.5, -2.0, 7.25]));
println!("largest of nothing: {:?}", largest::<u8>(&[]));
println!("mean of u8s: {:?}", mean(&[1_u8, 2, 4]));
println!("mean of f32s: {:?}", mean(&[0.5_f32, 1.5]));
```

```text
largest i32: Some(9)
largest f64: Some(7.25)
largest of nothing: None
mean of u8s: Some(2.3333333333333335)
mean of f32s: Some(1.0)
```

`largest::<u8>(&[])` needs the turbofish because an empty slice gives the compiler no hint about
`T`.

## 7.2 Iterators

An iterator is anything implementing:

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // ... dozens of provided methods built on `next`
}
```

Adaptors (`map`, `filter`, ...) wrap an iterator in another iterator and are **lazy**: nothing
happens until a consumer (`collect`, `sum`, `for`, `fold`, `find`...) pulls values.

| Obtain | Items | Meaning |
|---|---|---|
| `v.iter()` | `&T` | borrow |
| `v.iter_mut()` | `&mut T` | borrow and modify |
| `v.into_iter()` | `T` | consume |
| `.copied()` / `.cloned()` | `T` | turn `&T` into `T` for `Copy` / `Clone` types |

### From loop to pipeline

The Egyptian dates count regnal years in the reigns of the Babylonian kings. To convert one, you
add up the years of all earlier reigns. First with a loop:

```rust
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
```

And with iterators:

```rust
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
```

- `scan` is like `map` with a mutable state (here, the running total); returning `Some(x)` yields
  `x`, returning `None` would end the stream.
- `find` stops at the first match, like the early `return`.
- `.map` on an `Option` transforms the `Some` content.

Both agree for every reign (a test checks that):

```text
cumulated years:
Nabunasir        Some(14) Some(14)
Kandalanu        Some(122) Some(122)
Nebuchadnezzar   Some(186) Some(186)
Nobody           None None
```

Neither is "more correct". The iterator version has no mutable variable in your code and says
*what* is computed; the loop is sometimes clearer when the logic branches. Use whichever reads
best.

### More adaptors on the same table

```rust
let total: u32 = REIGNS.iter().map(|r| r.regnal_years).sum();

let long: Vec<_> = reigns_longer_than(20).map(|r| r.name).collect();

let (kingless, kings): (Vec<&Reign>, Vec<&Reign>) =
    REIGNS.iter().partition(|r| r.name.contains("Kingless"));

let longest = REIGNS.iter().max_by_key(|r| r.regnal_years).unwrap();

let first_three: Vec<_> = REIGNS.iter().take(3).map(|r| r.name).collect();
let backwards: Vec<_> = REIGNS.iter().rev().take(2).map(|r| r.name).collect();
```

```text
total regnal years: 186
longer than 20 years: ["Kandalanu", "Nabonpolassar", "Nebuchadnezzar"]
2 kingless periods, 15 reigns
longest: Nebuchadnezzar (43 years)
first three: ["Nabunasir", "Nabu-nadin-zeri", "Mukin-zeri and Pul"]
last two: ["Nebuchadnezzar", "Nabonpolassar"]
```

`collect` builds whatever collection the type annotation names (`Vec<_>`, `HashSet<_>`,
`String`, even `Result<Vec<_>, E>`: chapter 12). `partition` needs its result type spelled out,
because it could produce any pair of collections; without the annotation the compiler reports
"type annotations needed".

### Returning `impl Iterator`

```rust
fn reigns_longer_than(years: u32) -> impl Iterator<Item = &'static Reign> {
    REIGNS.iter().filter(move |r| r.regnal_years > years)
}
```

The actual return type is `Filter<slice::Iter<'static, Reign>, {closure}>`, and a closure's type
cannot be written down. `impl Iterator<Item = ...>` says "some iterator yielding these items"
without naming it. `move` is needed because the closure outlives the function's stack frame, so it
must own `years`. The real `Egyptian` module exposes `all_reigns()` in the same way.

### Floats are only `PartialOrd`

`NaN` is not equal to itself, so `f64` does not implement `Ord`, and `iter().max()` or `sort()`
do not compile for floats. Two idioms:

```rust
let ra = [18.6156, 16.4901, 13.4199, 10.1396];
let max = ra.iter().copied().fold(f64::NEG_INFINITY, f64::max);
let mut sorted = ra;
sorted.sort_by(|a, b| a.total_cmp(b));
println!("max RA {max}, sorted {sorted:?}");
```

```text
max RA 18.6156, sorted [10.1396, 13.4199, 16.4901, 18.6156]
```

`f64::max` is passed by name as the folding function: any function with a matching signature
can stand in for a closure. `total_cmp` defines a total order, in which `NaN` sorts after
everything.

## 7.3 Closures

A closure is an anonymous function that can capture variables from its surroundings:
`|x| x * factor`. The types of parameters and result are usually inferred.

```rust
/// Takes any closure `(f64, f64) -> (f64, f64)` and applies it pairwise.
fn map_pairs(first: &[f64], second: &[f64], f: impl Fn(f64, f64) -> (f64, f64))
    -> Vec<(f64, f64)>
{
    first.iter().zip(second).map(|(&a, &b)| f(a, b)).collect()
}

let dec = [38.78, -26.43, -11.16, 11.97];
let eps = 23.44_f64.to_radians();               // captured by reference
let out = map_pairs(&ra, &dec, |r, d| (r * 15.0, d + eps.sin()));
```

```text
 279.234   39.178
 247.352  -26.032
 201.298  -10.762
 152.094   12.368
```

(The `+ eps.sin()` has no astronomical meaning; it just shows a captured variable.)

`zip` pairs up two iterators and stops at the shorter one. The real `batch.rs` has a private helper
of the same name, `map_pairs`, which checks that both arrays have equal length and then applies one
scalar conversion to every element: the scalar code is written once and the batch functions are
three-line wrappers.

### `Fn`, `FnMut`, `FnOnce`

Every closure implements one or more of three traits, depending on what it does with its captures:

| Trait | Closure body... | May be called |
|---|---|---|
| `FnOnce` | moves a captured value out | once |
| `FnMut` | mutates a captured value | many times, needs `&mut` access |
| `Fn` | only reads (or captures nothing) | many times, even concurrently |

Every `Fn` is also `FnMut` and `FnOnce`. As parameters, ask for the *weakest* bound that works
(`FnOnce` if you call it once, as in `Checked::map`; `Fn` if you call it repeatedly and
the caller wants to share it).

```rust
/// Returns a closure that captured `factor` by move.
fn scaler(factor: f64) -> impl Fn(f64) -> f64 {
    move |x| x * factor
}

let to_degrees = scaler(15.0);
println!("6 h = {} deg", to_degrees(6.0));

let mut calls = 0;
let mut counted = |x: f64| {
    calls += 1;           // mutates a capture, so this closure is FnMut
    x.to_radians()
};
counted(90.0);
counted(180.0);
println!("closure called {calls} times");
```

```text
6 h = 90 deg
closure called 2 times
```

`calls` can be printed only after `counted` is no longer used: the closure holds a `&mut` borrow
of it (chapter 5).

### Different closures in one collection

Every closure has its own unique type, so a `Vec` of closures needs a trait object:

```rust
let pipeline: Vec<Box<dyn Fn(f64) -> f64>> = vec![
    Box::new(|x| x * 15.0),
    Box::new(|x| x + 360.0),
    Box::new(|x| x.rem_euclid(360.0)),
];
let result = pipeline.iter().fold(-6.0, |acc, f| f(acc));
println!("pipeline(-6 h) = {result}");
```

```text
pipeline(-6 h) = 270
```

-6 h is -90 degrees; adding 360 and reducing modulo 360 gives 270 degrees.

## 7.4 Choosing the tool

| You want | Use |
|---|---|
| the same code for several types | generics with bounds |
| a mixed collection of types | `Vec<Box<dyn Trait>>` |
| to transform or filter a sequence | iterator adaptors |
| to stop at the first match | `find`, `position`, `any`, `all` |
| to pass a small piece of behaviour | a closure, `impl Fn...` |
| complicated state in a loop | a plain `for` loop |

**In the real project:** [`star_coord.rs`](../../../ptolemy/crates/ptolemy-astro/src/star_coord.rs)
defines `Checked<T>` (line 87), and [`batch.rs`](../../../ptolemy/crates/ptolemy-astro/src/batch.rs)
the `map_pairs` helper that all batch conversions use.

## Exercises

1. Write `cumulated_years` a third time, with `position` and `take`/`sum`:
   `REIGNS.iter().position(|r| r.name == name)` gives the index, then sum the first `index + 1`
   reigns.
2. Write `fn smallest<T: PartialOrd + Copy>(items: &[T]) -> Option<T>`, then call it with a slice of
   `f64`. Add a `NaN`; what happens?
3. Implement `Checked<T>::zip<U>(self, other: Checked<U>) -> Checked<(T, U)>`, taking the smaller
   of the two gradients (`f64::min`).
4. Use `windows(2)` on the RA array to print the difference between neighbouring stars.
5. Write `fn compose(f: impl Fn(f64) -> f64, g: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64`.
6. Replace `Box<dyn Fn>` in the pipeline with a plain array of function pointers `[fn(f64) -> f64; 3]`.
   Why does this work only for closures that capture nothing?

## Summary

- Generics give one definition for many types, with no run-time cost; bounds say what the type can
  do.
- Iterator pipelines are lazy, composable, and compile to loops. `collect`, `sum` and friends
  consume them.
- Closures capture their environment; `Fn`/`FnMut`/`FnOnce` describe how.
- `f64` is not `Ord`: use `fold(f64::max)` and `total_cmp`.

Previous: [Chapter 6](../ch6/README.md) | Next: [Chapter 8: Modules and crates](../ch8/README.md)
