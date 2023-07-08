use crate::{Max, Min};
use std::ops::{Add, Deref, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SampleRate(u32);

impl SampleRate {
    pub fn new(sample_rate: impl Into<u32>) -> Self {
        SampleRate(sample_rate.into())
    }

    pub const fn new_const(sample_rate: u32) -> Self {
        SampleRate(sample_rate)
    }

    pub const fn get(&self) -> u32 {
        self.0
    }

    pub fn set(&mut self, sample_rate: impl Into<u32>) -> &mut Self {
        self.0 = sample_rate.into();

        self
    }

    pub const fn into_inner(self) -> u32 {
        self.0
    }
}

impl Deref for SampleRate {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Min for SampleRate {
    type Rhs = SampleRate;
    type Output = SampleRate;

    fn min(self, rhs: Self::Rhs) -> Self::Output {
        if self.get() < rhs.get() {
            self
        } else {
            rhs
        }
    }
}

impl Max for SampleRate {
    type Rhs = SampleRate;
    type Output = SampleRate;

    fn max(self, rhs: Self::Rhs) -> Self::Output {
        if self.get() > rhs.get() {
            self
        } else {
            rhs
        }
    }
}

impl Mul for SampleRate {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        SampleRate::from(self.get() * rhs.get())
    }
}

impl Mul<u32> for SampleRate {
    type Output = u32;

    fn mul(self, rhs: u32) -> Self::Output {
        self.get() * rhs
    }
}

impl Mul<SampleRate> for u32 {
    type Output = Self;

    fn mul(self, rhs: SampleRate) -> Self::Output {
        self * rhs.get()
    }
}

impl Add for SampleRate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        SampleRate::from(self.get() + rhs.get())
    }
}

impl Add<u32> for SampleRate {
    type Output = u32;

    fn add(self, rhs: u32) -> Self::Output {
        self.get() + rhs
    }
}

impl Add<SampleRate> for u32 {
    type Output = Self;

    fn add(self, rhs: SampleRate) -> Self::Output {
        self + rhs.get()
    }
}

impl Sub for SampleRate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        SampleRate::from(self.get() - rhs.get())
    }
}

impl Sub<u32> for SampleRate {
    type Output = u32;

    fn sub(self, rhs: u32) -> Self::Output {
        self.get() - rhs
    }
}

impl Sub<SampleRate> for u32 {
    type Output = Self;

    fn sub(self, rhs: SampleRate) -> Self::Output {
        self - rhs.get()
    }
}

impl From<u32> for SampleRate {
    fn from(sample_rate: u32) -> Self {
        Self::new(sample_rate)
    }
}

impl From<SampleRate> for u32 {
    fn from(sample_rate: SampleRate) -> Self {
        sample_rate.into_inner()
    }
}

#[cfg(feature = "dac")]
impl From<cpal::SampleRate> for SampleRate {
    fn from(sample_rate: cpal::SampleRate) -> Self {
        Self(sample_rate.0)
    }
}
