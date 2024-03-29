use crate::{Max, Min};
use std::ops::{Add, Deref, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NumChannels(usize);

impl NumChannels {
    pub fn new(num_channels: impl Into<usize>) -> Self {
        NumChannels(num_channels.into())
    }

    pub const fn get(&self) -> usize {
        self.0
    }

    pub fn set(&mut self, num_channels: impl Into<usize>) -> &mut Self {
        self.0 = num_channels.into();

        self
    }

    pub const fn into_inner(self) -> usize {
        self.0
    }
}

impl Deref for NumChannels {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl From<u16> for NumChannels {
    fn from(num_channels: u16) -> Self {
        Self::new(num_channels as usize)
    }
}

impl From<u32> for NumChannels {
    fn from(num_channels: u32) -> Self {
        Self::new(num_channels as usize)
    }
}

impl From<i32> for NumChannels {
    fn from(num_channels: i32) -> Self {
        Self::new(num_channels as usize)
    }
}

impl From<NumChannels> for usize {
    fn from(num_channels: NumChannels) -> Self {
        num_channels.into_inner()
    }
}
