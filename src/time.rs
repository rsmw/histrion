use ordered_float::NotNan;

use std::ops::Add;

#[derive(Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Instant(NotNan<f64>);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Interval(NotNan<f64>);

impl Instant {
    pub fn delta(self, other: Self) -> Interval {
        Interval(other.0 - self.0)
    }
}

impl Interval {
    pub fn one() -> Self {
        Interval(1.0.into())
    }

    pub fn from_f64(f: f64) -> Self {
        Interval(f.into())
    }
}

impl Add<Interval> for Instant {
    type Output = Self;

    fn add(self, Interval(i): Interval) -> Self {
        Instant(self.0 + i)
    }
}

impl From<Interval> for f64 {
    fn from(Interval(value): Interval) -> Self {
        value.into()
    }
}

impl From<Instant> for f64 {
    fn from(Instant(value): Instant) -> Self {
        value.into()
    }
}
