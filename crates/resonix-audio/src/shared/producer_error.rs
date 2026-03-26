use alloc::boxed::Box;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("Not enought space to write data to.")]
    InsufficientSpace,
    #[error("Unknown error occurred: {0:?}")]
    UnknownError(Box<dyn core::error::Error>),
}
