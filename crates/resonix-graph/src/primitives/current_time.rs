use core::ops::Deref;

/// Currently, this value aligns with the web_sys's BaseAudioContext::curent_time
/// implementation, but the specific type here is subject to change.
#[derive(Copy, Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct CurrentTime(f64);

impl CurrentTime {
    pub const fn new(time: f64) -> Self {
        Self(time)
    }
}

// other convenience implementations possible here

impl Deref for CurrentTime {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<f64> for CurrentTime {
    fn from(value: f64) -> Self {
        CurrentTime(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_deref() {
        let current_time = CurrentTime::new(512.0);
        assert_eq!(*current_time, 512.0);
    }

    #[test]
    fn from_f64_creates_current_time_with_that_value() {
        let current_time = CurrentTime::from(1024f64);
        assert_eq!(*current_time, 1024.0);
    }

    #[test]
    fn current_times_with_same_value_are_equal() {
        assert_eq!(CurrentTime::new(128.0), CurrentTime::new(128.0));
    }

    #[test]
    fn smaller_current_time_is_less_than_larger() {
        assert!(CurrentTime::new(64.0) < CurrentTime::new(128.0));
    }
}
