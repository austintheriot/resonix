use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum MockAudioOutputError {
    #[error("Error writing to output audio stream")]
    WriteError,
}
