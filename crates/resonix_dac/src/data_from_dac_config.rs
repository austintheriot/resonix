use std::sync::Arc;

use crate::DACConfig;

/// Allows a function to pull whatever data it needs out of the audio player context
/// and whatever user-specified data is specified inside
pub trait DataFromDACConfig {
    fn from_config(config: Arc<DACConfig>) -> Self;
}
