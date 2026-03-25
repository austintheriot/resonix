use alloc::boxed::Box;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("No space to write data to")]
    InsufficientSpace,
    #[error("Failed to write sample: {0:?}")]
    WriteFailure(Option<Box<dyn core::error::Error>>),
}
