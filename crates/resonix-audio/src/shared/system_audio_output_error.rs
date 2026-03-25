use alloc::boxed::Box;
use core::error::Error;

use thiserror::Error;

use crate::ProducerError;

#[derive(Error, Debug)]
pub enum SystemAudioOutputError {
    #[error("Producer error occurred: {0}")]
    ProducerError(#[from] ProducerError),
    #[error("Internal buffer error write occurred: {0}")]
    WriteError(#[from] Box<dyn Error>),
}
