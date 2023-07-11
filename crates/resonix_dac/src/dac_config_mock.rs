use crate::DACConfigBuildError;

/// when testing, mock functionality
pub struct DACConfig {
    pub num_channels: u16,
    pub sample_rate: u32,
    pub num_frames: u32,
}

impl DACConfig {
    pub fn from_defaults() -> Result<Self, DACConfigBuildError> {
        Ok(Self::default())
    }

    pub fn from_data(
        num_channels: u16,
        sample_rate: u32,
        num_frames: u32,
    ) -> Result<Self, DACConfigBuildError> {
        Ok(Self {
            num_channels,
            sample_rate,
            num_frames,
        })
    }

    pub fn num_channels(&self) -> u16 {
        self.num_channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn num_frames(&self) -> u32 {
        self.num_frames
    }
}

impl Default for DACConfig {
    fn default() -> Self {
        Self {
            num_channels: 2,
            sample_rate: 44100,
            num_frames: u32::MAX,
        }
    }
}
