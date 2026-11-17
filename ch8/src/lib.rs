//! Chapter 8: Modules and crates. A library split into files, with a re-exported public API.
//!
//! Layout:
//! ```text
//! src/lib.rs        crate root: declares modules and re-exports the public API
//! src/units.rs      crate-private constants
//! src/calendar.rs   leap years and month tables
//! src/duration.rs   Duration
//! src/main.rs       a separate binary crate that uses this library
//! ```

pub mod calendar;
pub mod duration;
mod units; // private: not part of the public API

pub use duration::{Duration, Sign};

/// Items defined directly in the crate root are reachable as `ch8::version()`.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
