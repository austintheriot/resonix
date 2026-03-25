use alloc::boxed::Box;
use core::error::Error;

use thiserror::Error;

use crate::ConsumerError;

#[derive(Error, Debug)]
pub enum SystemAudioInputError {
    #[error("Consumer error occurred: {0}")]
    ConsumerError(#[from] ConsumerError),
    #[error("Internal buffer error read occurred: {0}")]
    ReadError(#[from] Box<dyn Error>),
}
