# Chapter 9: Numerics: angles, obliquity and precession

This chapter is the astronomy at the heart of Ptolemy: the angle between the celestial equator and
the ecliptic (the *obliquity*), conversion of star coordinates between the two systems, and
*precession*, the slow rotation of the equinoxes which Hipparchus discovered and Ptolemy
used to relate star longitudes of his day to older observations. It is also a chapter about
floating-point numbers, where Rust makes the pitfalls explicit.

**You will learn**

- `f64` behaviour: rounding, `NaN`, `%` versus `rem_euclid`
- normalising angles, and the edge case of `rem_euclid` returning exactly `360.0`
- the Julian Day formula for both calendars
- polynomial series in Julian centuries (mean obliquity, nutation)
- `atan2`, `asin`, `clamp`, `sin_cos`, `to_radians`
- checking results against published examples (Meeus)
- precession with the three rotation angles zeta, z, theta, and how to use it

Code: [`src/main.rs`](src/main.rs). Run it with `cargo run -p ch9`.

## 9.1 Floating point

```rust
println!("0.1 + 0.2 == 0.3 ? {}", 0.1 + 0.2 == 0.3);
println!("|0.1 + 0.2 - 0.3| = {:e}", (0.1_f64 + 0.2 - 0.3).abs());
println!("-7 % 360       = {}", -7.0_f64 % 360.0);
println!("-7 rem_euclid  = {}", (-7.0_f64).rem_euclid(360.0));
println!("tiny negative  = {}", (-1e-20_f64).rem_euclid(360.0));
println!("normalize_deg  = {}", normalize_deg(-1e-20));
println!("angle_diff(359, 1) = {}", angle_diff(359.0, 1.0));
println!("sqrt(-1) is NaN: {}", (-1.0_f64).sqrt().is_nan());
```

```text
0.1 + 0.2 == 0.3 ? false
|0.1 + 0.2 - 0.3| = 5.551115123125783e-17
-7 % 360       = -7
-7 rem_euclid  = 353
tiny negative  = 360
normalize_deg  = 0
angle_diff(359, 1) = -2
sqrt(-1) is NaN: true
```

Rules of thumb:

- `f64` has about 15 to 16 significant decimal digits. Never compare results with `==`; compare
  within a tolerance, for instance `(a - b).abs() < 1e-9` (the `approx` crate does it, chapter 10).
- `%` keeps the sign of the dividend, so `-7.0 % 360.0` is `-7.0`. For wrapping angles you want
  `rem_euclid`, whose result is always in `[0, 360)`.
- Except for one edge: `(-1e-20).rem_euclid(360.0)` is mathematically `360 - 1e-20`, which rounds to
  exactly `360.0` in `f64`. So the real code and the chapter code guard it:

```rust
/// Maps any angle into `[0, 360)`.
fn normalize_deg(angle: f64) -> f64 {
    let wrapped = angle.rem_euclid(360.0);
    // rem_euclid can return exactly 360.0 for tiny negative inputs.
    if wrapped >= 360.0 { 0.0 } else { wrapped }
}
```

- Operations that are mathematically invalid return `NaN` (not a panic, not an error), and `NaN`
  then propagates silently through every later calculation. Check `is_nan()` at boundaries.
  Integer overflow, by contrast, panics in debug builds.
- Angles are in degrees at the project's API and in radians inside `sin`/`cos`:
  `x.to_radians()` and `x.to_degrees()` convert.

### The shortest signed difference

```rust
/// Signed shortest difference `a - b` in degrees, in `[-180, 180)`.
fn angle_diff(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}
```

`359 - 1` is `358`, but the longitudes are only `-2` degrees apart across the 0/360 seam.
Comparing angles with a plain subtraction is a classic bug. This function is what the tests use to
compare right ascensions that may be `359.9999999` versus `0.0000001`.

## 9.2 Julian Day

The Julian Day (JD) is a continuous count of days from noon on 1 January 4713 BC (Julian calendar),
and the clock of all astronomical formulas. Meeus, *Astronomical Algorithms*, chapter 7:

```rust
fn julian_day(year: i32, month: i32, day: f64, gregorian: bool) -> f64 {
    let (y, m) = if month <= 2 { (year - 1, month + 12) } else { (year, month) };
    let b = if gregorian {
        let a = (f64::from(y) / 100.0).floor();
        2.0 - a + (a / 4.0).floor()
    } else {
        0.0                       // Julian calendar: no correction
    };
    (365.25 * f64::from(y + 4716)).floor() + (30.6001 * f64::from(m + 1)).floor()
        + day + b - 1524.5
}
```

- January and February are treated as months 13 and 14 of the *previous* year. The century term
  `a` must use that adjusted year `y`. The Python original used the unadjusted year, which gave
  wrong results for January and February of centennial years such as 1900 or 2000; the Rust port
  fixes it (see "Differences from the Python original" in the real README).
- The `gregorian` flag selects the calendar of the **input date**. Dates up to 1582-10-04 are Julian;
  dates from 1582-10-15 are Gregorian. Ptolemy's epoch (year 134) is a Julian date.
- `f64::from(y)` is a lossless `i32` to `f64` conversion; `as f64` would work too, but `from`
  cannot silently lose precision, so prefer it when it compiles.

The real project computes the JD with integer arithmetic (chapter 4), which is exact for any date.
This chapter's float version is the textbook formula and is enough for the examples.

## 9.3 Solar and lunar parameters

Everything depends on `T`, the time in Julian centuries since J2000.0 (JD 2451545.0 = 2000-01-01
12:00):

```rust
const J2000: f64 = 2_451_545.0;
const DAYS_PER_CENTURY: f64 = 36_525.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct AstroParams { jd: f64, t: f64 }

impl AstroParams {
    fn from_jd(jd: f64) -> Self {
        Self { jd, t: (jd - J2000) / DAYS_PER_CENTURY }
    }
}
```

The quantities are low-order polynomials in `T`, written in the textbook form (not Horner form) so that each
term can be compared against the book:

```rust
/// Mean obliquity of the ecliptic (Meeus 22.2). The cubic term must be divided by 3600.
fn mean_obliquity(&self) -> f64 {
    let t = self.t;
    23.439_291_111 - 46.815 * t / 3600.0 - 0.000_59 * t * t / 3600.0
        + 0.001_813 * t * t * t / 3600.0
}
```

Meeus gives the coefficients in arcseconds, so each must be divided by 3600 to get degrees. The
Python original divided the last term by 60, an error of a factor 60 that is invisible for dates near the
present (where `T` is small) and large for Ptolemy's epoch (T = -18). Number literals can contain
underscores for readability.

The nutation, a small periodic wobble of the axis with a period of 18.6 years, uses the longitude
of the Moon's ascending node `omega` and the mean longitudes of the Sun and Moon (Meeus ch. 22,
low-accuracy series):

```rust
fn nutation_longitude(&self) -> f64 {
    let omega = self.moon_ascending_node_longitude();
    let sun = self.sun_mean_longitude();
    let moon = self.moon_mean_longitude();
    let arcsec = -17.2 * omega.to_radians().sin()
        - 1.32 * (2.0 * sun).to_radians().sin()
        - 0.23 * (2.0 * moon).to_radians().sin()
        + 0.21 * (2.0 * omega).to_radians().sin();
    arcsec / 3600.0
}
```

and the *true* obliquity is `mean_obliquity() + nutation_obliquity()`.

Checking against Meeus example 22.a (1987-04-10 00:00 TD):

```rust
let params = AstroParams::from_jd(julian_day(1987, 4, 10.0, true));
```

```text
JD 2446895.5 T = -0.127296372348
mean obliquity   23.440946 deg
nutation (lon)   -3.863 arcsec
nutation (obl)   9.468 arcsec
equation of centre 1.9042 deg
```

The book gives `JD = 2446895.5`, `T = -0.127296372348`, a mean obliquity of 23°26'27.407" =
23.440946° and a nutation of -3.788" in longitude and +9.443" in obliquity. Our nutation differs by
0.075" and 0.025": the book's own low-accuracy series is only good to about 0.5", so the test
accepts a 0.1" tolerance and says so in a comment:

```rust
// The low-accuracy series is good to about 0.1 arcsecond.
close(p.nutation_longitude() * 3600.0, -3.788, 0.1);
```

Choosing and justifying a tolerance, instead of loosening it until the test passes, is part of
numerical work.

## 9.4 Equatorial and ecliptic coordinates

A star's **equatorial** coordinates are right ascension `ra` (measured along the celestial equator
from the March equinox, in degrees or hours: 24 h = 360 degrees) and declination `dec` (north or
south of the equator). Its **ecliptic** coordinates are longitude `lon` and latitude `lat`, measured
along and from the apparent path of the Sun. Ptolemy's catalogue uses ecliptic coordinates, modern
catalogues use equatorial ones. The conversion is a rotation by the obliquity `eps`:

$$
\tan\lambda = \frac{\sin\alpha\cos\varepsilon + \tan\delta\,\sin\varepsilon}{\cos\alpha},
\qquad
\sin\beta = \sin\delta\cos\varepsilon - \cos\delta\sin\varepsilon\sin\alpha
$$

```rust
fn equatorial_to_ecliptic_at(eq: Equatorial, obliquity_deg: f64) -> Ecliptic {
    let (sin_a, cos_a) = eq.ra_deg.to_radians().sin_cos();
    let (sin_d, cos_d) = eq.dec_deg.to_radians().sin_cos();
    let (sin_e, cos_e) = obliquity_deg.to_radians().sin_cos();
    let y = sin_a * cos_e * cos_d + sin_d * sin_e;
    let x = cos_a * cos_d;
    // `asin` is only defined on [-1, 1]; rounding can push the value just outside.
    let sin_beta = (sin_d * cos_e - cos_d * sin_e * sin_a).clamp(-1.0, 1.0);
    Ecliptic {
        lon_deg: normalize_deg(y.atan2(x).to_degrees()),
        lat_deg: sin_beta.asin().to_degrees(),
    }
}
```

Three numerical idioms:

- **`atan2(y, x)`** instead of `atan(y / x)`: it knows the quadrant, handles `x == 0` and returns
  an angle in `(-pi, pi]`, which `normalize_deg` maps to `[0, 360)`. The longitude is
  `atan2(sin a cos e + tan d sin e, cos a)`; multiplying `y` and `x` by `cos d` keeps it finite near
  the poles where `tan d` blows up.
- **`clamp(-1.0, 1.0)`** before `asin`. For a star at the pole the value can be `1.0000000000000002`
  by rounding, and `asin` of that is `NaN`. Clamping is an intentional, local fix of
  rounding error.
- **`sin_cos()`** returns both at once and reads better than two calls.

The reverse conversion has the same structure with the sign of the `sin e` terms flipped.

Meeus example 13.a: Pollux, `ra = 116.328942`, `dec = +28.026183`, `eps = 23.4392911`:

```text
Pollux Equatorial { ra_deg: 116.328942, dec_deg: 28.026183 }
  -> Ecliptic { lon_deg: 113.21562957874626, lat_deg: 6.684169796279848 }
  -> Equatorial { ra_deg: 116.32894200000001, dec_deg: 28.026183000000003 }
```

The book's results are 113.215630 and 6.684170, and the round trip returns the input to within
`2e-14` degrees. Note how the last line prints `116.32894200000001`: rounding noise, which is why
tests compare with tolerances.

## 9.5 Precession

The Earth's axis traces a circle of about 26,000 years, so the equinoxes move westwards along the
ecliptic at roughly 50" a year (1 degree per 72 years). A catalogue of equatorial positions (RA,
dec) refers to the equator and equinox *of its epoch*. To compare Ptolemy's stars of year 134 with
the J2000 catalogue you rotate the coordinates by three angles zeta, z and theta (Meeus ch. 21, eq. 21.2):

```rust
struct EquatorialPrecession { zeta: f64, z: f64, theta: f64 }

impl EquatorialPrecession {
    /// `big_t`: centuries from J2000 to the data epoch; `t`: centuries from data epoch to target.
    fn new(big_t: f64, t: f64) -> Self { /* three polynomials, in arcseconds / 3600 */ }

    fn between(data_jd: f64, target_jd: f64) -> Self {
        Self::new(
            (data_jd - J2000) / DAYS_PER_CENTURY,
            (target_jd - data_jd) / DAYS_PER_CENTURY,
        )
    }

    fn apply(&self, eq: Equatorial) -> Equatorial {
        let (sin_t, cos_t) = self.theta.to_radians().sin_cos();
        let (sin_d, cos_d) = eq.dec_deg.to_radians().sin_cos();
        let (sin_az, cos_az) = (eq.ra_deg + self.zeta).to_radians().sin_cos();
        let a = cos_d * sin_az;
        let b = cos_t * cos_d * cos_az - sin_t * sin_d;
        let c = (sin_t * cos_d * cos_az + cos_t * sin_d).clamp(-1.0, 1.0);
        Equatorial {
            ra_deg: normalize_deg(a.atan2(b).to_degrees() + self.z),
            dec_deg: c.asin().to_degrees(),
        }
    }
}
```

The structure is a plain struct holding the three angles; constructing it once and applying it to
many stars avoids recomputing the polynomials. A test checks that precessing over a zero interval is
the identity. `normalize_deg` on the result is one of the fixes against the Python original, whose
precessed right ascensions could be negative or above 360.

### The Almagest epoch

Putting it all together: take J2000 catalogue positions of three stars, precess them back to the
epoch of Ptolemy's observations (1 January 134, Julian calendar, noon), and convert to ecliptic
coordinates using the obliquity *of that epoch*:

```rust
let j2000 = julian_day(2000, 1, 1.5, true);
let ptolemy = julian_day(134, 1, 1.5, false);        // Julian calendar date
let obliquity = AstroParams::from_jd(ptolemy).obliquity();
let rotation = EquatorialPrecession::between(j2000, ptolemy);
```

```text
JD(134-01-01 12:00 Julian) = 1770002
EquatorialPrecession { zeta: -11.956944504483419, z: -11.880637098219475, theta: -10.354573805005236 }
Epoch 134: ecliptic longitude / latitude (degrees)
Vega      259.4836   61.9675
Antares   223.8092   -4.3388
Spica     177.8897   -1.9520
```

Sanity checks you can do without any reference:

- The interval is 1866 years, i.e. `t = -18.66` centuries. Zeta and z are about `2306" * t`, which is
  `-43,000"`, or `-11.9` degrees; theta is about `2004" * t`, which is `-10.4` degrees. All three are
  negative because we go *backwards* in time, and the numbers above match.
- Spica's ecliptic latitude is close to 0 (it is almost on the ecliptic, which is why it was so useful
  for measuring precession). Its longitude of 177.9 degrees can be compared with Ptolemy's catalogue
  value of Virgo 26 deg 40' = 176.67 degrees: the difference of about 1.2 degrees is the known
  systematic offset of the Almagest catalogue. The Almagest and its errors are a rich field for
  exercises, and for the real `examples/almagest_stars.rs`.

The real example `almagest_stars` prints the same table for a longer list of stars, and in chapter
12 you call it from your own code.

## 9.6 Summary of numerical advice

| Problem | Remedy |
|---|---|
| `==` on floats | compare with a tolerance |
| angles wrap | `rem_euclid(360.0)`, then guard the 360.0 edge; `angle_diff` for differences |
| `asin`/`acos` domain | `clamp(-1.0, 1.0)` |
| quadrant of `atan` | `atan2` |
| unit mix-ups | degrees at the API, radians inside; hours only at the input edge |
| unit mix-ups in polynomials | write coefficients in the units of the source and convert once (`/ 3600.0`) |
| trusting formulas | test against published values (Meeus) with a justified tolerance |
| `NaN` | check `is_nan()` at the boundary |

Chapter 10 returns to unit and property testing; chapter 11 vectorises these functions over arrays
of stars.

**In the real project:** see [`angles.rs`](../../../ptolemy/crates/ptolemy-astro/src/angles.rs)
(`normalize_deg`, the `Equatorial` and `Ecliptic` types with `from_hours`, `RaUnit`),
[`astro_params.rs`](../../../ptolemy/crates/ptolemy-astro/src/astro_params.rs) (the full set of
parameters, including `moon_umbral_cone` and `true_longitude`) and
[`star_coord.rs`](../../../ptolemy/crates/ptolemy-astro/src/star_coord.rs) (conversions, nutation,
precession and the derivatives). The list "Differences from the Python original" in
[`README.md`](../../../ptolemy/README.md) records every numerical bug fixed during the port.

## Exercises

1. Show that `normalize_deg(angle)` is idempotent: write a loop test over many angles checking
   `normalize_deg(normalize_deg(a)) == normalize_deg(a)`.
2. Write `fn hours_to_degrees(h: f64) -> f64` and `fn degrees_to_hms(d: f64) -> (u32, u32, f64)`.
   What happens for 359.99999999 degrees?
3. Compute the JD of 1900-01-15 12:00 with `julian_day` and compare it with the integer algorithm of
   chapter 4. Then try the unadjusted century term to see the Python bug.
4. Print `mean_obliquity()` for the years -1000, 0, 1000, 2000, 3000. By how much does it change per
   century?
5. Precess Spica from J2000 to the epoch of Hipparchus (-128). How much has its longitude changed
   compared with 134?
6. Replace `clamp` in `equatorial_to_ecliptic_at` by nothing and search for an input where the result
   becomes `NaN`. (Hint: dec = 90 degrees.)

## Summary

- Floats are approximations: compare with tolerances, wrap angles with `rem_euclid`, guard the
  `360.0` edge.
- `atan2`, `clamp` before `asin`, and consistent units are the keys to reliable trigonometry.
- Astronomical quantities are polynomials in Julian centuries since J2000.
- Verify every formula against a published worked example.

Previous: [Chapter 8](../ch8/README.md) | Next: [Chapter 10: Testing](../ch10/README.md)
