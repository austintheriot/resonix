#[cfg(feature = "mock")]
mod mock_audio_output;
#[cfg(feature = "mock")]
pub use mock_audio_output::*;
#[cfg(feature = "mock")]
mod mock_audio_input;
#[cfg(feature = "mock")]
pub use mock_audio_input::*;

// errors should not leak
#[cfg(feature = "mock")]
mod mock_audio_input_error;
#[cfg(feature = "mock")]
mod mock_audio_output_error;
#[cfg(feature = "mock")]
pub(crate) use mock_audio_input_error::*;
#[cfg(feature = "mock")]
pub(crate) use mock_audio_output_error::*;
