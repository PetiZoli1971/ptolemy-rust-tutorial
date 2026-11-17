# Chapter 8: Modules and crates

So far each chapter was a single `main.rs`. Real programs are split into files, with a deliberate
boundary between what is public and what is an internal detail. This chapter turns the `Duration`
and calendar code of earlier chapters into a small library crate, and then looks at how the real
Ptolemy project is organised as a Cargo workspace of two libraries.

**You will learn**

- packages, crates, modules, and how files map to modules
- `pub`, `pub(crate)` and private items; privacy rules
- `use`, `crate::`, `super::`, `self::` paths; re-exports with `pub use`
- a package with both a library and a binary
- unit tests, integration tests (`tests/`) and doctests
- Cargo workspaces, shared dependencies and lints

Code: [`src/lib.rs`](src/lib.rs), [`src/main.rs`](src/main.rs),
[`src/calendar.rs`](src/calendar.rs), [`src/duration.rs`](src/duration.rs),
[`src/units.rs`](src/units.rs), [`tests/api.rs`](tests/api.rs).
Run it with `cargo run -p ch8`; test it with `cargo test -p ch8`.

## 8.1 Vocabulary

| Term | Meaning |
|---|---|
| **package** | what `Cargo.toml` describes; contains one or more crates |
| **crate** | one compilation unit: a library (`src/lib.rs`) or a binary (`src/main.rs`, `src/bin/*.rs`) |
| **module** | a namespace inside a crate; `mod name;` |
| **workspace** | several packages that share one `Cargo.lock` and one `target/` directory |

## 8.2 The layout of `ch8`

```text
ch8/
  Cargo.toml
  src/
    lib.rs        crate root of the library: declares modules, re-exports the API
    units.rs      crate-private constants
    calendar.rs   leap years and month tables
    duration.rs   Duration and Sign
    main.rs       a separate binary crate that uses the library
  tests/
    api.rs        integration tests, which only see the public API
```

Because the package has both `src/lib.rs` and `src/main.rs`, Cargo builds two crates: a library
named `ch8` and a binary that depends on it. This is the standard way to make a program testable:
keep the logic in the library, and keep `main.rs` thin.

### The crate root

```rust
//! Chapter 8: Modules and crates. A library split into files, with a re-exported public API.

pub mod calendar;
pub mod duration;
mod units; // private: not part of the public API

pub use duration::{Duration, Sign};

/// Items defined directly in the crate root are reachable as `ch8::version()`.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- `mod calendar;` tells the compiler to load the module from `calendar.rs` (or `calendar/mod.rs`).
  A module that is not declared is not compiled at all, even if the file exists.
- `pub mod` makes the module visible to users of the crate; plain `mod` keeps it private.
- `pub use duration::{Duration, Sign};` **re-exports**: users can write `ch8::Duration` instead of
  `ch8::duration::Duration`. The internal file layout is then free to change without breaking them.
- `env!("CARGO_PKG_VERSION")` is replaced at compile time by the version from `Cargo.toml`.
- `//!` documents the enclosing item (here, the crate); `///` documents the next item.

## 8.3 Visibility

Everything is **private by default**: visible inside its own module and in the child modules of it.

| Written | Visible to |
|---|---|
| (nothing) | the current module and its children |
| `pub(super)` | the parent module |
| `pub(crate)` | everything in this crate, nothing outside |
| `pub` | everybody who can reach the module |

```rust
// src/units.rs
pub(crate) const SECS_PER_MINUTE: i64 = 60;
pub(crate) const SECS_PER_HOUR: i64 = 3_600;
pub(crate) const SECS_PER_DAY: i64 = 86_400;
pub(crate) const SECS_PER_YEAR: i64 = 365 * SECS_PER_DAY;
```

These constants are shared by `duration.rs` and `calendar.rs`, but they are no business of the
library's users.

```rust
// src/duration.rs
use std::fmt;

use crate::units::{SECS_PER_DAY, SECS_PER_HOUR, SECS_PER_MINUTE, SECS_PER_YEAR};

/// The field is private to this module: outside code must use the constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    total_secs: i64,
}
```

- `crate::` starts a path at the crate root. `super::` goes to the parent module, `self::` stays
  in the current one.
- `use` brings names into scope; it does not make anything more visible.
- A struct can be `pub` while its fields are private. This is what makes `Duration` an
  abstraction: no code outside `duration.rs` can build a `Duration` with a nonsensical
  `total_secs`, or depend on the field existing. If it later becomes a `(years, days, ...)`
  struct, nobody is affected.

A private function in a module is visible to that module's own tests (`mod tests` is a child):

```rust
// src/calendar.rs
pub fn days_in_year(year: i32) -> u16 {
    year_length(is_leap_year(year))
}

fn year_length(leap: bool) -> u16 {            // private helper
    if leap { 366 } else { 365 }
}

#[cfg(test)]
mod tests {
    use super::*;                               // everything from the parent, even private items

    #[test]
    fn private_helpers_are_visible_to_child_modules() {
        assert_eq!(year_length(true), 366);
    }
}
```

`#[cfg(test)]` means the module is only compiled for `cargo test`. `use super::*;` is the standard
first line of a test module.

## 8.4 Using the library from the binary

```rust
// src/main.rs
use ch8::calendar::{days_in_month, is_leap_year};
use ch8::{Duration, Sign};

fn main() {
    println!("ch8 version {}", ch8::version());
    println!("1900 leap? {}", is_leap_year(1900));
    println!("Feb 2000: {:?} days", days_in_month(2000, 2));
    println!("{}", ch8::calendar::days_in_year(2024));

    let d = Duration::new(Sign::Plus, 3, 100, 0, 0, 0);
    println!("{d} = {} s", d.total_secs());
}
```

```text
ch8 version 0.1.0
1900 leap? false
Feb 2000: Some(29) days
366
+3 years = 103248000 s
```

`main.rs` is a *client* of the library: it uses `ch8::...`, not `crate::...`, and sees only
the public items. Two things it cannot do, shown as comments in the source and reproducible by
uncommenting them:

```rust
let d = Duration { total_secs: 5 };
// error[E0451]: field `total_secs` of struct `Duration` is private

println!("{}", ch8::units::SECS_PER_DAY);
// error[E0603]: module `units` is private
```

Notice the `Display` output only prints whole years: `3 years + 100 days` shows as `+3 years`. It
is a deliberately small type; the full one is in chapter 6 and in the real project.

## 8.5 Tests in three places

| Kind | Where | Can see | Run by |
|---|---|---|---|
| unit test | `#[cfg(test)] mod tests` in the same file | private items | `cargo test` |
| integration test | `tests/*.rs`; each file is a separate crate | public API only | `cargo test` |
| doctest | code blocks inside `///` comments | public API only | `cargo test` |

An integration test uses the crate like any external user:

```rust
// tests/api.rs
use ch8::{Duration, Sign};

#[test]
fn ordering_uses_the_signed_total() {
    let neg = Duration::new(Sign::Minus, 1, 0, 0, 0, 0);
    let pos = Duration::new(Sign::Plus, 1, 0, 0, 0, 0);
    assert!(neg < pos);
}
```

A doctest is example code in the documentation which is also run as a test, so documentation
cannot go out of date:

```rust
/// Julian rule up to and including 1582, Gregorian rule afterwards.
///
/// ```
/// use ch8::calendar::is_leap_year;
/// assert!(is_leap_year(1500));
/// assert!(!is_leap_year(1900));
/// ```
pub fn is_leap_year(year: i32) -> bool { /* ... */ }
```

`cargo doc --open -p ch8` builds the HTML documentation from the `///` and `//!` comments, and
shows only the public items.

Chapter 10 treats testing in depth.

## 8.6 Workspaces: the real project

Chapters 1 to 12 of this tutorial are themselves a workspace: the root
[`Cargo.toml`](../Cargo.toml) lists them all as `members`, so `cargo test --workspace` tests
everything and all share one `target/` directory.

The real Ptolemy project has the same idea with two *library* crates. Its root manifest
([`Cargo.toml`](../../../ptolemy/Cargo.toml)) shows the features of a workspace manifest:

```toml
[workspace]
resolver = "2"
members = ["crates/*"]            # every directory under crates/

[workspace.package]               # fields that member packages can inherit
version = "0.1.0"
edition = "2021"
license = "MIT"
publish = false

[workspace.dependencies]          # one place to choose versions
approx = "0.5"
ndarray = "0.16"
thiserror = "2"
ptolemy-time  = { path = "crates/ptolemy-time" }
ptolemy-astro = { path = "crates/ptolemy-astro" }

[workspace.lints.rust]            # one place to configure lints
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
```

and a member opts in with `workspace = true`:

```toml
# crates/ptolemy-astro/Cargo.toml
[package]
name = "ptolemy-astro"
version.workspace = true
edition.workspace = true

[dependencies]
ptolemy-time.workspace = true     # a path dependency on the sibling crate
ndarray.workspace = true
thiserror.workspace = true

[dev-dependencies]                # only for tests, examples and benches
approx.workspace = true
proptest.workspace = true

[lints]
workspace = true
```

Design points worth copying:

- **One-way dependency.** `ptolemy-astro` depends on `ptolemy-time` (it needs `HistoricDateTime`
  for the Julian Day), never the reverse. Time handling can be used without any astronomy code, and
  the two compile separately.
- **A small public API per crate.** Each `lib.rs` is a list of `pub mod` and `pub use`
  re-exports:

  ```rust
  // ptolemy-time/src/lib.rs
  pub mod calendar;
  pub mod duration;
  pub mod egyptian;
  pub mod error;
  pub mod historic_datetime;

  pub use duration::{Duration, Sign};
  pub use error::{DateError, DurationError, JdError};
  pub use historic_datetime::HistoricDateTime;
  ```

  The most-used types are available as `ptolemy_time::HistoricDateTime`, and the rest through
  their module.
- **`unsafe_code = "forbid"`.** The whole workspace is safe Rust, and the compiler enforces it.
- **Crate names** use hyphens in `Cargo.toml` (`ptolemy-time`) and underscores in code
  (`ptolemy_time`).

## Exercises

1. Move `Sign` into its own file `sign.rs`, and keep `ch8::Sign` working via `pub use`. Do the
   tests still compile unchanged? (They should.)
2. Add `pub(crate) fn is_valid_month(month: u8) -> bool` to `calendar` and use it from
   `days_in_month`. Try to call it from `main.rs`: what is the error?
3. Re-export `calendar::days_in_year` at the crate root (`pub use calendar::days_in_year;`) and
   write a doctest for it. Then run `cargo test -p ch8 --doc`.
4. Add `src/bin/leap.rs` printing whether the year given as the first argument is a leap year, and
   run it with `cargo run -p ch8 --bin leap -- 1900`.
5. Create a third crate in the workspace that depends on `ch8` through
   `ch8 = { path = "../ch8" }`.
6. In the real workspace, run `cargo tree -p ptolemy-astro` and identify which dependencies are
   shared with `ptolemy-time`.

## Summary

- Packages contain crates; crates contain modules; `mod` declares them, files hold them.
- Everything is private by default; `pub(crate)` is the useful middle ground; private fields make
  abstractions that can change.
- `pub use` shapes the public API independently of the file layout.
- Put logic in `lib.rs` and keep `main.rs` thin; unit tests live beside the code, integration tests
  in `tests/`, doctests in comments.
- A workspace shares the lock file, `target/`, dependency versions and lints among its crates.

Previous: [Chapter 7](../ch7/README.md) | Next: [Chapter 9: Numerics: angles, obliquity, precession](../ch9/README.md)
