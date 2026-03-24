use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CpalAudioOutputError {
    #[error("Error writing to output audio stream")]
    WriteError,
}
