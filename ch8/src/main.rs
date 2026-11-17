use ch8::calendar::{days_in_month, is_leap_year};
use ch8::{Duration, Sign};

// `use` paths start at the crate name for a binary that depends on a library.
// This does not compile: `total_secs` is private.
//
//     let d = Duration { total_secs: 5 };
//     error[E0451]: field `total_secs` of struct `Duration` is private

fn main() {
    println!("ch8 version {}", ch8::version());
    println!("1900 leap? {}", is_leap_year(1900));
    println!("Feb 2000: {:?} days", days_in_month(2000, 2));
    println!("{}", ch8::calendar::days_in_year(2024));

    let d = Duration::new(Sign::Plus, 3, 100, 0, 0, 0);
    println!("{d} = {} s", d.total_secs());

    // println!("{}", ch8::units::SECS_PER_DAY); // error[E0603]: module `units` is private
}
