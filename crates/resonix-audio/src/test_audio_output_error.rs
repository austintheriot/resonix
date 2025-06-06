use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum TestAudioOutputError {
    #[error("Error writing to output audio stream")]
    WriteError,
}
