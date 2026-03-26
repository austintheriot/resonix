use alloc::boxed::Box;
use core::error::Error;

use thiserror::Error;

use crate::ConsumerError;

#[derive(Error, Debug)]
pub enum SystemAudioInputError {
    #[error("Consumer error occurred: {0:?}")]
    ConsumerError(#[from] ConsumerError),
    #[error("Unknown error occurred: {0:?}")]
    UnknownError(#[from] Box<dyn Error>),
}
