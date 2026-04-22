use core::ops::Deref;

#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleRate(u32);

impl Default for SampleRate {
    fn default() -> Self {
        Self(44100)
    }
}

impl SampleRate {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

// other convenience implementations possible here

impl Deref for SampleRate {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for SampleRate {
    fn from(value: i32) -> Self {
        SampleRate(value as u32)
    }
}

impl From<u32> for SampleRate {
    fn from(value: u32) -> Self {
        SampleRate(value)
    }
}

impl From<usize> for SampleRate {
    fn from(value: usize) -> Self {
        SampleRate(value as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_44100() {
        assert_eq!(*SampleRate::default(), 44100);
    }

    #[test]
    fn new_stores_value_accessible_via_deref() {
        let sample_rate = SampleRate::new(512);
        assert_eq!(*sample_rate, 512);
    }

    #[test]
    fn from_usize_creates_sample_rate_with_that_value() {
        let sample_rate = SampleRate::from(1024usize);
        assert_eq!(*sample_rate, 1024);
    }

    #[test]
    fn from_i32_casts_to_usize() {
        let sample_rate = SampleRate::from(256i32);
        assert_eq!(*sample_rate, 256);
    }

    #[test]
    fn sample_rates_with_same_value_are_equal() {
        assert_eq!(SampleRate::new(128), SampleRate::new(128));
    }

    #[test]
    fn smaller_sample_rate_is_less_than_larger() {
        assert!(SampleRate::new(64) < SampleRate::new(128));
    }
}
