#[cfg(feature = "cpal")]
mod cpal_audio_output;
#[cfg(feature = "cpal")]
pub use cpal_audio_output::*;
#[cfg(feature = "cpal")]
mod cpal_audio_input;
#[cfg(feature = "cpal")]
pub use cpal_audio_input::*;

// errors should not leak
#[cfg(feature = "cpal")]
mod cpal_audio_output_error;
#[cfg(feature = "cpal")]
pub(crate) use cpal_audio_output_error::*;
#[cfg(feature = "cpal")]
mod cpal_audio_input_error;
#[cfg(feature = "cpal")]
pub(crate) use cpal_audio_input_error::*;
