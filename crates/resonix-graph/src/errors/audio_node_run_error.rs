use thiserror::Error;

use crate::primitives::AudioBufferError;

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
