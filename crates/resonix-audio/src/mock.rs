#[cfg(feature = "mock")]
mod mock_audio_output;
#[cfg(feature = "mock")]
pub use mock_audio_output::*;

// errors should not leak
#[cfg(feature = "mock")]
mod mock_audio_output_error;
#[cfg(feature = "mock")]
pub(crate) use mock_audio_output_error::*;
