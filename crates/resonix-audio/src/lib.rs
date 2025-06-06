#![no_std]

extern crate alloc;

#[cfg(feature = "cpal")]
mod cpal_audio_output;
#[cfg(feature = "cpal")]
pub use cpal_audio_output::*;
#[cfg(feature = "cpal")]
mod cpal_audio_output_error;
#[cfg(feature = "cpal")]
pub(crate) use cpal_audio_output_error::*;

#[cfg(feature = "test")]
mod test_audio_output;
#[cfg(feature = "test")]
pub use test_audio_output::*;
#[cfg(feature = "test")]
mod test_audio_output_error;
#[cfg(feature = "test")]
pub(crate) use test_audio_output_error::*;

mod system_audio_output;
mod system_audio_output_error;
pub use system_audio_output::*;
pub use system_audio_output_error::*;
