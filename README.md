# Ptolemy by Example

[Rust](https://www.rust-lang.org/) is a systems programming language focused on safety, speed and
concurrency. *Ptolemy by Example* teaches Rust in the style of
[Rust By Example](https://doc.rust-lang.org/rust-by-example/): short, runnable programs, each
introducing a few concepts. The difference is that every example comes from one real project, the
`ptolemy` workspace, a Rust port of a Python astronomy package for the geometry of Ptolemy's
*Almagest*.

By the end you will have rebuilt, piece by piece, the code that turns a historic date such as
*"Hadrian, year 17, 20 Payni, 23:15"* into a Julian Day, and a star catalogue position such as
*"Vega, J2000"* into the ecliptic longitude it had when Ptolemy observed it.

The real project lives in [`../../ptolemy`](../../ptolemy) (that is `~/_my/rust/ptolemy`). The
tutorial never modifies it. Chapters 1-11 are self-contained crates; chapter 12 depends on the real
crates by path.

## Chapters

| # | Chapter | Rust topics | Project code it is based on |
|---|---|---|---|
| 1 | [Hello, Ptolemy](ch1/README.md) | Cargo, `println!`, primitives, `const`, shadowing, tuples, arrays | Julian Day, J2000 |
| 2 | [Calendar rules](ch2/README.md) | functions, expressions, `if`, `match`, `for`/`while`/`loop`, `Option` | `calendar.rs` |
| 3 | [Custom types](ch3/README.md) | `struct`, `enum`, `impl`, `derive`, privacy | `Equatorial`, `RaUnit`, `EgyptianMonth`, `Duration` |
| 4 | [Error handling](ch4/README.md) | `Result`, `?`, `thiserror`, `From`, `let else`, `main` returning `Result` | `DateError`, `JdError`, `HistoricDateTime::new` |
| 5 | [Ownership, borrowing, lifetimes](ch5/README.md) | move, `Clone`/`Copy`, `&`, `&mut`, `'a`, `'static` | `Reign`, `Egyptian`, star catalogues |
| 6 | [Traits](ch6/README.md) | `Display`, operators, `Ord`, `From`, `FromStr`, your own traits, `dyn` | `Duration`, `EgyptianMonth` |
| 7 | [Generics, iterators, closures](ch7/README.md) | `<T>`, bounds, `where`, `Iterator`, `impl Fn`, `Box<dyn Fn>` | `Checked<T>`, `cumulated_years`, `map_pairs` |
| 8 | [Modules and crates](ch8/README.md) | `mod`, `pub`, `pub(crate)`, `use`, re-exports, library + binary, workspaces | the workspace layout |
| 9 | [Numerics: angles, obliquity, precession](ch9/README.md) | `f64`, `rem_euclid`, `atan2`, `clamp`, float pitfalls | `angles.rs`, `astro_params.rs`, `star_coord.rs` |
| 10 | [Testing](ch10/README.md) | unit, integration, doc tests, `approx`, golden files, `proptest` | the `tests/` directories |
| 11 | [External crates: ndarray and log](ch11/README.md) | dependencies, `ndarray`, `log`, a custom logger | `batch.rs`, `Checked` conversions |
| 12 | [Putting it together](ch12/README.md) | path dependencies, workspace tooling, docs, clippy | `examples/almagest_stars.rs` |

If you know Rust By Example, the mapping is roughly: chapter 1 = *Hello World* and *Primitives*,
2 = *Flow of Control* and *Functions*, 3 = *Custom Types*, 4 = *Error handling*,
5 = *Scoping rules*, 6 = *Traits*, 7 = *Generics* and *Closures*/*Iterators*, 8 = *Modules*,
*Crates*, *Cargo*, 10 = *Testing*, 11 = *Std library types* and external crates.

## Requirements

- Rust stable with edition 2021 (written with Cargo 1.99). Install it with
  [rustup](https://www.rust-lang.org/tools/install).
- The crates `approx`, `log`, `ndarray`, `proptest`, `serde_json` and `thiserror`. The committed
  `Cargo.lock` is copied from the real project, so `--offline` works if they are already in
  `~/.cargo/registry`.
- Chapter 12 needs the real project checked out at `~/_my/rust/ptolemy`.

## How to use this tutorial

Every chapter directory is a Cargo package named after the directory (`ch1`, `ch2`, ...), and all of
them form one Cargo workspace.

```sh
cd ~/_my/rust/tutorial/ptolemy

cargo run  -p ch3              # run the program of chapter 3
cargo test -p ch3              # run its tests
cargo test --workspace         # everything
cargo clippy --workspace --all-targets -- -D warnings
```

A chapter directory contains:

| File | Purpose |
|---|---|
| `README.md` | The lesson: concepts, annotated code, program output, exercises |
| `Cargo.toml` | Package manifest (dependencies, if any) |
| `src/main.rs` (or `src/lib.rs`) | The complete, runnable code of the lesson |
| `tests/` | Integration tests (chapters 8 and 10) |

Read the README first, then open `src/main.rs` next to it, run it, and change something. Each
README ends with exercises; try them in the chapter's own crate. Code blocks in the READMEs are
excerpts from the crate; their output blocks are the real output of `cargo run`.

Conventions used in the READMEs:

- `// error[E0382]: ...` after a commented-out line shows the compiler error you get if you
  enable it.
- **In the real project** boxes link to the file in `~/_my/rust/ptolemy` that the chapter is based
  on, so you can compare the simplified tutorial code with the production code.
- Angles are always in degrees unless stated otherwise, and years use astronomical numbering
  (year 0 exists and is 1 BC).

## The domain in two minutes

You do not need to know astronomy; every chapter explains the little it uses.

- **Julian Day (JD)** is a continuous count of days since noon on -4712-01-01. It turns messy
  calendar arithmetic into a subtraction. J2000.0, noon on 2000-01-01, is JD 2 451 545.0.
- **Calendar reform.** Dates up to 1582-10-04 are in the Julian calendar; the next day is
  1582-10-15 in the Gregorian calendar. The ten days between never existed.
- **Egyptian calendar.** 12 months of 30 days plus 5 extra days, 365 days with no leap years. The
  *Almagest* dates observations as "year Y of king K, day D of month M".
- **Coordinates.** *Equatorial* coordinates (right ascension, declination) are tied to the Earth's
  equator; *ecliptic* coordinates (longitude, latitude) are tied to the Sun's yearly path. They
  differ by a rotation by the obliquity (about 23.4 degrees).
- **Precession** is the slow wobble of the Earth's axis, about 1 degree per 72 years, which shifts
  star longitudes between Ptolemy's epoch and ours.

## Where to go next

After chapter 12, read the project's [PLAN.md](../../ptolemy/PLAN.md) for the TDD plan and the list
of bugs fixed relative to the Python original, and then try the planned phase 9: a `ptolemy-cli`
crate on top of `ptolemy-astro`.
