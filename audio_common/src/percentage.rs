use crate::{max::Max, min::Min};
use std::ops::{Add, Mul, Sub};

#[derive(PartialEq, PartialOrd, Default, Clone, Debug, Copy)]
pub struct Percentage(f32);

/// Only accepts values ranging from 0.0 -> 1.0.
///
/// If a value outside of these bounds is received, it will be truncated to match the range.
impl Percentage {
    pub const PERCENTAGE_MAX: f32 = 1.0;

    pub const PERCENTAGE_MIN: f32 = 0.0;

    pub fn sanitize_percentage(percentage: f32) -> f32 {
        percentage
            .max(Self::PERCENTAGE_MIN.into())
            .min(Self::PERCENTAGE_MAX.into())
    }

    pub fn new(percentage: f32) -> Self {
        Percentage(Self::sanitize_percentage(percentage))
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn set(&mut self, percentage: f32) -> &mut Self {
        self.0 = Self::sanitize_percentage(percentage);

        self
    }
}

impl Min for Percentage {
    type Rhs = Percentage;
    type Output = Percentage;

    fn min(self, rhs: Self::Rhs) -> Self::Output {
        if self.get() < rhs.get() {
            self
        } else {
            rhs
        }
    }
}

impl Max for Percentage {
    type Rhs = Percentage;
    type Output = Percentage;

    fn max(self, rhs: Self::Rhs) -> Self::Output {
        if self.get() > rhs.get() {
            self
        } else {
            rhs
        }
    }
}

impl Mul for Percentage {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Percentage::from(self.get() * rhs.get())
    }
}

impl Mul<f32> for Percentage {
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        self.get() * rhs
    }
}

impl Mul<Percentage> for f32 {
    type Output = Self;

    fn mul(self, rhs: Percentage) -> Self::Output {
        self * rhs.get()
    }
}

impl Add for Percentage {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Percentage::from(self.get() + rhs.get())
    }
}

impl Add<f32> for Percentage {
    type Output = f32;

    fn add(self, rhs: f32) -> Self::Output {
        self.get() + rhs
    }
}

impl Add<Percentage> for f32 {
    type Output = Self;

    fn add(self, rhs: Percentage) -> Self::Output {
        self + rhs.get()
    }
}

impl Sub for Percentage {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Percentage::from(self.get() - rhs.get())
    }
}

impl Sub<f32> for Percentage {
    type Output = f32;

    fn sub(self, rhs: f32) -> Self::Output {
        self.get() - rhs
    }
}

impl Sub<Percentage> for f32 {
    type Output = Self;

    fn sub(self, rhs: Percentage) -> Self::Output {
        self - rhs.get()
    }
}

impl From<f32> for Percentage {
    fn from(percentage: f32) -> Self {
        Self::new(percentage)
    }
}
