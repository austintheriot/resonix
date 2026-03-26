use alloc::boxed::Box;
use core::error::Error;

use thiserror::Error;

use crate::ProducerError;

#[derive(Error, Debug)]
pub enum SystemAudioOutputError {
    #[error("Producer error occurred: {0:?}")]
    ProducerError(#[from] ProducerError),
    #[error("Unknown error occurred: {0:?}")]
    UnknownError(#[from] Box<dyn Error>),
}
