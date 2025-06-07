use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CpalAudioInputError {
    #[error("Error reading from input audio stream")]
    ReadError,
}
