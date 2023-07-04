use crate::DACConfigBuildError;

/// when testing, mock functionality
pub struct DACConfig {
    num_channels: u16,
    sample_rate: u32,
}

impl DACConfig {
    pub fn from_defaults() -> Result<Self, DACConfigBuildError> {
        Ok(Self::default())
    }

    pub fn from_data(num_channels: u16, sample_rate: u32) -> Result<Self, DACConfigBuildError> {
        Ok(Self {
            num_channels,
            sample_rate,
        })
    }

    pub fn num_channels(&self) -> u16 {
        2
    }

    pub fn sample_rate(&self) -> u32 {
        44100
    }
}

impl Default for DACConfig {
    fn default() -> Self {
        Self {
            num_channels: 2,
            sample_rate: 44100,
        }
    }
}
