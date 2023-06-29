#[cfg(all(test, feature = "mock_dac"))]

/// when testing, mock functionality
pub struct DACConfig;

impl DACConfig {
    pub fn from_defaults() -> Result<Self, ()> {
        Ok(Self)
    }

    pub fn num_channels(&self) -> u32 {
        2
    }

    pub fn sample_rate(&self) -> u32 {
        44100
    }
}
