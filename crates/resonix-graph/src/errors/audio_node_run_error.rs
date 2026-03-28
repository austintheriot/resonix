use thiserror::Error;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::errors::AudioBufferError;

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Error, Debug)]
pub enum AudioNodeRunError {
    #[error("too many inputs (expected {expected:?}, found {found:?})")]
    TooManyInputs { expected: usize, found: usize },
    #[error("audio buffer error: {0:?}")]
    AudioBuffer(AudioBufferError),
}

impl From<AudioBufferError> for AudioNodeRunError {
    fn from(e: AudioBufferError) -> Self {
        Self::AudioBuffer(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn too_many_inputs_display_shows_expected_and_found_counts() {
        let error = AudioNodeRunError::TooManyInputs {
            expected: 2,
            found: 5,
        };
        assert_eq!(error.to_string(), "too many inputs (expected 2, found 5)");
    }

    #[test]
    fn audio_buffer_error_display_wraps_inner_error() {
        let inner = AudioBufferError::ZeroChannels;
        let error = AudioNodeRunError::AudioBuffer(inner);
        assert!(error.to_string().contains("ZeroChannels"));
    }

    #[test]
    fn from_audio_buffer_error_wraps_into_audio_buffer_variant() {
        let inner = AudioBufferError::ZeroChannels;
        let error: AudioNodeRunError = inner.into();
        assert!(matches!(
            error,
            AudioNodeRunError::AudioBuffer(AudioBufferError::ZeroChannels)
        ));
    }

    #[test]
    fn from_audio_buffer_error_with_length_mismatch_preserves_fields() {
        let inner = AudioBufferError::LengthChannelMismatch {
            len: 3,
            channels: 2,
        };
        let error: AudioNodeRunError = inner.clone().into();
        assert!(matches!(
            error,
            AudioNodeRunError::AudioBuffer(AudioBufferError::LengthChannelMismatch {
                len: 3,
                channels: 2
            })
        ));
    }
}
