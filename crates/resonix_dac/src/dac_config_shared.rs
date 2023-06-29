use std::sync::Arc;

use cpal::DefaultStreamConfigError;
use thiserror::Error;

use crate::{DACConfig, DataFromDACConfig};

#[derive(Error, Debug)]
pub enum DACConfigBuildError {
    #[error("no audio output devices found")]
    NooOutputDevicesAvailable,
    #[error("no default stream config available. original error: {0:?}")]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),
}

impl DataFromDACConfig for Arc<DACConfig> {
    fn from_config(config: Arc<DACConfig>) -> Self {
        Arc::clone(&config)
    }
}
