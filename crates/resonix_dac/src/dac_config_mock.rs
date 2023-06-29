use crate::DACConfigBuildError;

/// when testing, mock functionality
pub struct DACConfig;

impl DACConfig {
    pub fn from_defaults() -> Result<Self, DACConfigBuildError> {
        Ok(Self)
    }

    pub fn num_channels(&self) -> u16 {
        2
    }

    pub fn sample_rate(&self) -> u32 {
        44100
    }
}
