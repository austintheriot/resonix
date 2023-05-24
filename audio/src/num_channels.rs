use crate::{max::Max, min::Min};
use std::ops::{Add, Mul, Sub};

#[derive(PartialEq, PartialOrd, Default, Clone, Debug, Copy)]
pub struct NumChannels(usize);

/// Only accepts values ranging from 0.0 -> 1.0.
///
/// If a value outside of these bounds is received, it will be truncated to match the range.
impl NumChannels {
    pub const NUM_CHANNELS_MAX: usize = 500;

    pub const NUM_CHANNELS_MIN: usize = 1;

    pub fn sanitize_channels(num_channels: impl Into<usize>) -> usize {
        num_channels.into().clamp(Self::NUM_CHANNELS_MIN, Self::NUM_CHANNELS_MAX)
    }

    pub fn new(num_channels: impl Into<usize>) -> Self {
        NumChannels(Self::sanitize_channels(num_channels))
    }

    pub const fn get(&self) -> usize {
        self.0
    }

    pub fn set(&mut self, num_channels: impl Into<usize>) -> &mut Self {
        self.0 = Self::sanitize_channels(num_channels.into());

        self
    }

    pub const fn into_inner(self) -> usize {
        self.0
    }
}

impl Min for NumChannels {
    type Rhs = NumChannels;
    type Output = NumChannels;

    fn min(self, rhs: Self::Rhs) -> Self::Output {
        if self.get() < rhs.get() {
            self
        } else {
            rhs
        }
    }
}

impl Max for NumChannels {
    type Rhs = NumChannels;
    type Output = NumChannels;

    fn max(self, rhs: Self::Rhs) -> Self::Output {
        if self.get() > rhs.get() {
            self
        } else {
            rhs
        }
    }
}

impl Mul for NumChannels {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        NumChannels::from(self.get() * rhs.get())
    }
}

impl Mul<usize> for NumChannels {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        self.get() * rhs
    }
}

impl Mul<NumChannels> for usize {
    type Output = Self;

    fn mul(self, rhs: NumChannels) -> Self::Output {
        self * rhs.get()
    }
}

impl Add for NumChannels {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        NumChannels::from(self.get() + rhs.get())
    }
}

impl Add<usize> for NumChannels {
    type Output = usize;

    fn add(self, rhs: usize) -> Self::Output {
        self.get() + rhs
    }
}

impl Add<NumChannels> for usize {
    type Output = Self;

    fn add(self, rhs: NumChannels) -> Self::Output {
        self + rhs.get()
    }
}

impl Sub for NumChannels {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        NumChannels::from(self.get() - rhs.get())
    }
}

impl Sub<usize> for NumChannels {
    type Output = usize;

    fn sub(self, rhs: usize) -> Self::Output {
        self.get() - rhs
    }
}

impl Sub<NumChannels> for usize {
    type Output = Self;

    fn sub(self, rhs: NumChannels) -> Self::Output {
        self - rhs.get()
    }
}

impl From<usize> for NumChannels {
    fn from(num_channels: usize) -> Self {
        Self::new(num_channels)
    }
}


impl From<NumChannels> for usize {
    fn from(num_channels: NumChannels) -> Self {
        num_channels.into_inner()
    }
}
