use alloc::boxed::Box;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("Not enought space to write data to.")]
    InsufficientSpace,
    #[error("Failed to write value: {0:?}")]
    WriteError(Box<dyn core::error::Error>),
}
