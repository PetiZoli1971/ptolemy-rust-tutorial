use ch8::{Duration, Sign};

#[test]
fn ordering_uses_the_signed_total() {
    let neg = Duration::new(Sign::Minus, 1, 0, 0, 0, 0);
    let pos = Duration::new(Sign::Plus, 1, 0, 0, 0, 0);
    assert!(neg < pos);
}

#[test]
fn display_shows_sign_and_years() {
    assert_eq!(
        Duration::new(Sign::Minus, 2, 10, 0, 0, 0).to_string(),
        "-2 years"
    );
}
