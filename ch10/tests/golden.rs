//! Integration test: uses only the public API, like a user of the crate. Compares with golden data.

use approx::assert_abs_diff_eq;
use ch10::julian_day;
use serde_json::Value;

const GOLDEN: &str = include_str!("golden.json");

#[test]
fn julian_day_matches_golden_values() {
    let golden: Value = serde_json::from_str(GOLDEN).expect("golden.json must be valid JSON");
    let cases = golden["jd"].as_array().expect("`jd` must be an array");
    assert!(!cases.is_empty());

    for case in cases {
        let d: Vec<i64> = case["date"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap())
            .collect();
        let expected = case["jd"].as_f64().unwrap();
        let got = julian_day(
            d[0] as i32,
            d[1] as u8,
            d[2] as u8,
            d[3] as u8,
            d[4] as u8,
            d[5] as u8,
        );
        assert_abs_diff_eq!(got, expected, epsilon = 1e-6);
    }
}

#[test]
fn julian_day_is_continuous_over_the_gregorian_reform() {
    let last_julian = julian_day(1582, 10, 4, 0, 0, 0);
    let first_gregorian = julian_day(1582, 10, 15, 0, 0, 0);
    assert_abs_diff_eq!(first_gregorian - last_julian, 1.0, epsilon = 1e-9);
}
