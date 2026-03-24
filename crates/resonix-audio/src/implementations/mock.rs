mod mock_audio_output;
pub use mock_audio_output::*;
mod mock_audio_input;
pub use mock_audio_input::*;

// errors should not leak
mod mock_audio_input_error;
mod mock_audio_output_error;
pub(crate) use mock_audio_input_error::*;
pub(crate) use mock_audio_output_error::*;
