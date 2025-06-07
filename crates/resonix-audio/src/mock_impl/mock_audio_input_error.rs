use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum MockAudioInputError {
    #[error("Error reading from input audio stream")]
    ReadError,
}
