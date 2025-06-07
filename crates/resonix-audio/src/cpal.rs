#[cfg(feature = "cpal")]
mod cpal_audio_output;
#[cfg(feature = "cpal")]
pub use cpal_audio_output::*;

// errors should not leak
#[cfg(feature = "cpal")]
mod cpal_audio_output_error;
#[cfg(feature = "cpal")]
pub(crate) use cpal_audio_output_error::*;
