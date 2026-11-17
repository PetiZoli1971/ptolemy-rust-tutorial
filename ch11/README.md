# Chapter 11: External crates: ndarray and log

Rust's standard library is small on purpose; most functionality comes from crates published on
[crates.io](https://crates.io). The Python original used `numpy` for arrays and the `logging`
module for diagnostics. The port uses `ndarray` and `log`, plus `thiserror` which you already
know from chapter 4.

**You will learn**

- adding dependencies with Cargo, version requirements, `Cargo.lock`
- `ndarray`: `Array1`, `ArrayView1`, `array!`, elementwise arithmetic, `mapv`, slicing with `s![]`, `Zip`
- building a batch API on top of a scalar function, and keeping the two consistent
- the *facade* pattern of the `log` crate, and writing your own logger backend
- using the conversion-quality indicator

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch11`.

## 11.1 Dependencies

```toml
[dependencies]
log = "0.4"
ndarray = "0.16"
thiserror = "2"
```

(In this workspace the versions are in the root `Cargo.toml` and the package says
`log.workspace = true`; chapter 8.)

- `"0.16"` means "any version compatible with 0.16", i.e. `>=0.16.0, <0.17.0`. For versions below
  1.0 the *minor* number is the breaking one; from `1.0` on it is the *major* number.
- The first `cargo build` downloads, compiles and caches the crates (and *their* dependencies). The exact
  versions chosen are recorded in `Cargo.lock`; commit it for applications, so that everybody builds
  with the same code.
- `cargo add ndarray`, `cargo update`, `cargo tree` and `cargo doc --open` (which also builds the
  documentation of every dependency) are the everyday commands. Look at docs.rs for any crate's API.
- Features: `ndarray = { version = "0.16", features = ["rayon"] }` switches on optional parts.

## 11.2 `ndarray` in five minutes

`ndarray` provides n-dimensional arrays. The 1-dimensional `Array1<f64>` is a heap-allocated vector
of numbers with numpy-like operations; `ArrayView1<f64>` is a borrowed, read-only view of one
(like `&[f64]` is to `Vec<f64>`, chapter 5).

```rust
use ndarray::{array, s, Array1, ArrayView1, Zip};

let ra_hours = array![18.615_607_22, 16.490_129_86, 13.419_890_15];
let ra_deg = &ra_hours * 15.0;               // elementwise arithmetic
let dec = array![38.782_993_11, -26.431_946_08, -11.161_244_91];
println!("ra_deg = {ra_deg}");
println!("sum = {:.3}, mean = {:.3}", ra_deg.sum(), ra_deg.mean().unwrap());
println!("first two = {}", ra_deg.slice(s![..2]));
println!("sines = {:.4}", dec.mapv(|d: f64| d.to_radians().sin()));
println!("linspace = {}", Array1::linspace(0.0, 360.0, 5));
```

```text
ra_deg = [279.2341083, 247.3519479, 201.29835225]
sum = 727.884, mean = 242.628
first two = [279.2341083, 247.3519479]
sines = [0.6264, -0.4451, -0.1936]
linspace = [0, 90, 180, 270, 360]
```

| Expression | Meaning |
|---|---|
| `array![1.0, 2.0]` | construct from literal values |
| `Array1::zeros(n)`, `Array1::linspace(a, b, n)` | filled arrays |
| `&a * 15.0`, `&a + &b` | elementwise operators; with references so `a` is not consumed |
| `a.sum()`, `a.mean()` | reductions; `mean()` returns `Option` because an empty array has no mean |
| `a.mapv(f)` | new array with `f` applied to each value (`map` gives `&T`, `mapv` gives `T`) |
| `a.slice(s![..2])` | a view of the first two elements; `s![1..;2]` takes every second |
| `a.view()` | an `ArrayView1` of the whole array |
| `a[i]` | indexing; panics out of range |
| `Zip::from(&mut o).and(&x).for_each(...)` | walk several arrays in lockstep |

Why `&ra_hours * 15.0` and not `ra_hours * 15.0`? Both work: the second *consumes* `ra_hours`
(reusing its memory), the first borrows. Chapter 5 again.

## 11.3 The batch layer

The scalar function `equatorial_to_ecliptic_at(Equatorial, f64) -> Ecliptic` from chapter 9 is
the single implementation of the maths. The batch version applies it to whole arrays. Since all
batch conversions are "two arrays in, two arrays out", one helper does the walking and the checking:

```rust
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
enum BatchError {
    #[error("input arrays have different lengths: {first} and {second}")]
    LengthMismatch { first: usize, second: usize },
}

type Pair = (Array1<f64>, Array1<f64>);

/// Applies `f` to two arrays element by element, after checking that they match in length.
fn map_pairs(
    first: ArrayView1<f64>,
    second: ArrayView1<f64>,
    f: impl Fn(f64, f64) -> (f64, f64),
) -> Result<Pair, BatchError> {
    if first.len() != second.len() {
        return Err(BatchError::LengthMismatch { first: first.len(), second: second.len() });
    }
    let mut out_first = Array1::zeros(first.len());
    let mut out_second = Array1::zeros(first.len());
    Zip::from(&mut out_first)
        .and(&mut out_second)
        .and(first)
        .and(second)
        .for_each(|o1, o2, &a, &b| (*o1, *o2) = f(a, b));
    Ok((out_first, out_second))
}

/// Returns `(longitudes, latitudes)` in degrees.
fn equatorial_to_ecliptic(
    ra_deg: ArrayView1<f64>,
    dec_deg: ArrayView1<f64>,
    obliquity_deg: f64,
) -> Result<Pair, BatchError> {
    map_pairs(ra_deg, dec_deg, |ra, dec| {
        let e = equatorial_to_ecliptic_at(Equatorial { ra_deg: ra, dec_deg: dec }, obliquity_deg);
        (e.lon_deg, e.lat_deg)
    })
}
```

Points:

- Parameters are `ArrayView1`: callers can pass whole arrays (`a.view()`), slices of arrays, or
  columns of a matrix, without copying. This is the numpy `ndarray` convention.
- A mismatch between the lengths is a recoverable `Err`, not a panic; the Python version
  raised an exception from deep inside a loop.
- The closure is generic (`impl Fn`), so the compiler inlines it into the loop: no cost.
- The destructuring assignment `(*o1, *o2) = f(a, b)` writes both outputs at once (stable since
  Rust 1.59).

```rust
let (lon, lat) = equatorial_to_ecliptic(ra_deg.view(), dec.view(), 23.44).unwrap();
println!("lon = {lon:.4}\nlat = {lat:.4}");

let short = array![1.0, 2.0];
match equatorial_to_ecliptic(ra_deg.view(), short.view(), 23.44) {
    Ok(_) => unreachable!(),
    Err(e) => println!("error: {e}"),
}
```

```text
lon = [285.3155, 249.7623, 203.8414]
lat = [61.7329, -4.5692, -2.0541]
error: input arrays have different lengths: 3 and 2
```

The most valuable test of a batch layer is "batch equals scalar":

```rust
#[test]
fn batch_equals_scalar_elementwise() {
    let ra = Array1::linspace(0.0, 359.0, 50);
    let dec = Array1::linspace(-80.0, 80.0, 50);
    let (lon, lat) = equatorial_to_ecliptic(ra.view(), dec.view(), 23.44).unwrap();
    for i in 0..ra.len() {
        let e = equatorial_to_ecliptic_at(Equatorial { ra_deg: ra[i], dec_deg: dec[i] }, 23.44);
        assert_eq!(lon[i], e.lon_deg);     // exact: same code, same inputs
        assert_eq!(lat[i], e.lat_deg);
    }
}
```

Exact `assert_eq!` on floats is legitimate here, since both sides run the very same function.
There is also a test for empty arrays and one for mismatched lengths.

**In the real project:** [`batch.rs`](../../../ptolemy/crates/ptolemy-astro/src/batch.rs) has exactly this
`map_pairs`, with `equatorial_to_ecliptic`, `ecliptic_to_equatorial` and `precession_high` as
three-line wrappers. Its `BatchError` also has a `Jd(#[from] JdError)` variant for the
functions that take a date. The old `StarCoord_Numpy` class of the Python version, which duplicated
the maths with numpy, no longer exists.

## 11.4 The `log` facade

Libraries should not decide where diagnostic messages go (a console? a file? a GUI?). The `log`
crate is a **facade**: it provides the macros `error!`, `warn!`, `info!`, `debug!`, `trace!` and the
`Log` trait, and the *application* installs one backend. If nobody installs one, the messages cost
almost nothing and go nowhere.

| Level | Use |
|---|---|
| `error!` | something failed |
| `warn!` | something suspicious, but handled |
| `info!` | high-level progress |
| `debug!` | detail for developers |
| `trace!` | everything |

The usual backend is `env_logger` (`RUST_LOG=debug cargo run`), but to see how little a backend is
(and to need no extra dependency) this chapter writes one:

```rust
use log::{Level, LevelFilter, Log, Metadata, Record};

struct StdoutLogger;

impl Log for StdoutLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{:<5}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: StdoutLogger = StdoutLogger;

fn main() {
    log::set_logger(&LOGGER).expect("logger is set once");
    log::set_max_level(LevelFilter::Info);
    // ...
}
```

- `set_logger` needs a `&'static dyn Log`; a `static` item has the `'static` lifetime (chapter 5).
  It can be called only once, hence `expect`.
- `set_max_level` is a cheap global filter checked by the macros before they even build the
  message.
- The macros use format-string syntax like `println!`, including inline arguments (`{name}`).

In the real project the library calls `log::warn!` and the *application* (the `almagest_stars`
example, a future CLI) installs a logger or none. The Python version required a `logging.conf`
file to exist; the Rust version needs none.

## 11.5 Conversion quality

Converting between equatorial and ecliptic coordinates amplifies or damps errors of the input,
depending on where in the sky the star is. The Python code computed a *gradient* of the
longitude with respect to the right ascension, and warned when it was large or small. The port
keeps that, returning the quality together with the value as a type rather than only logging it:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConversionQuality { Normal, ErrorAmplified, ErrorDamped }

impl ConversionQuality {
    fn from_gradient(gradient: f64) -> Self {
        if gradient.abs() > 1.5 {
            ConversionQuality::ErrorAmplified
        } else if gradient.abs() < 0.5 {
            ConversionQuality::ErrorDamped
        } else {
            ConversionQuality::Normal
        }
    }
}

fn equatorial_to_ecliptic_checked(name: &str, eq: Equatorial, obliquity_deg: f64)
    -> (Ecliptic, ConversionQuality)
{
    let g = equatorial_to_ecliptic_gradient(eq, obliquity_deg);
    let quality = ConversionQuality::from_gradient(g);
    match quality {
        ConversionQuality::Normal => log::debug!("{name}: gradient {g:.3}"),
        ConversionQuality::ErrorAmplified => {
            log::warn!("{name}: errors are amplified (gradient {g:.3})")
        }
        ConversionQuality::ErrorDamped => {
            log::warn!("{name}: errors are damped (gradient {g:.3})")
        }
    }
    (equatorial_to_ecliptic_at(eq, obliquity_deg), quality)
}
```

Running it:

```text
[INFO ] converting 3 stars
[WARN ] Vega: errors are amplified (gradient -2.022)
Vega        285.315   61.733 ErrorAmplified
equinox       0.000    0.000 Normal
[WARN ] far north: errors are damped (gradient 0.293)
far north    67.903   49.991 ErrorDamped
```

The `debug!` line for the equinox is filtered out by `set_max_level(Info)`, which the program proves
by not printing it.

Two design observations:

- The *caller* now gets the information in a type it can `match` on, not just in a log text. The
  library logging is a convenience.
- The gradient formula is ported unchanged from the Python original, and has **not** been independently
  verified; the real README says so. Porting a formula faithfully and verifying it are different
  jobs, and it is honest to mark which one was done. The thresholds 1.5 and 0.5 are tested
  (`quality_thresholds`).

## Exercises

1. Run `cargo tree -p ch11` and `cargo tree -p ch11 -i serde_json` (the latter prints "did not
   match"; why?).
2. Change `set_max_level(LevelFilter::Info)` to `Debug` and compare the output.
3. Write `ecliptic_to_equatorial` in the batch layer using `map_pairs`, with a test for the round trip
   to within `1e-9`.
4. What does `equatorial_to_ecliptic` return for two empty arrays? Is that right? Check with the
   `empty_arrays_give_empty_results` test.
5. Install `env_logger` (`cargo add env_logger -p ch11`, needs network access) and replace the custom logger by
   `env_logger::init()`. Run with `RUST_LOG=debug`.
6. Use `Zip::from(&lon).and(&lat).map_collect(|a, b| ...)` to compute the great-circle distance of every
   star to the point `(0, 0)`.

## Summary

- Add crates with `cargo add`, record exact versions in `Cargo.lock`, read documentation on
  docs.rs.
- `ndarray` gives numpy-like arrays; use views for parameters and write the maths once, scalar.
- `log` separates producing messages (libraries) from consuming them (applications).
- Return diagnostics as values (`ConversionQuality`) and log them as a convenience.

Previous: [Chapter 10](../ch10/README.md) | Next: [Chapter 12: Putting it together](../ch12/README.md)
