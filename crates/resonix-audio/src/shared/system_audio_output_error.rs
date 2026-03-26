use thiserror::Error;

use crate::ProducerError;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SystemAudioOutputError {
    #[error("Producer error occurred: {0:?}")]
    ProducerError(#[from] ProducerError),
}
