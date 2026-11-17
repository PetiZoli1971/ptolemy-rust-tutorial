//! Property tests: assert a law for many generated inputs instead of a few hand-picked ones.

use ch10::{angle_diff, julian_day_number, normalize_deg};
use proptest::prelude::*;

proptest! {
    #[test]
    fn normalize_is_always_in_range(x in -1.0e6f64..1.0e6) {
        let n = normalize_deg(x);
        prop_assert!((0.0..360.0).contains(&n), "{n}");
    }

    #[test]
    fn angle_diff_is_in_range_and_antisymmetric(a in 0.0f64..360.0, b in 0.0f64..360.0) {
        let d = angle_diff(a, b);
        prop_assert!((-180.0..=180.0).contains(&d));
        // The sign flips, except exactly at the +-180 boundary.
        prop_assert!((d + angle_diff(b, a)).abs() < 1e-9 || (d.abs() - 180.0).abs() < 1e-9);
    }

    #[test]
    fn consecutive_days_differ_by_one(year in -4000i32..3000, month in 1u8..=12, day in 1u8..=27) {
        // The ten skipped days of 1582 would break the property.
        prop_assume!(!(year == 1582 && month == 10));
        let a = julian_day_number(year, month, day);
        let b = julian_day_number(year, month, day + 1);
        prop_assert_eq!(b - a, 1);
    }
}
