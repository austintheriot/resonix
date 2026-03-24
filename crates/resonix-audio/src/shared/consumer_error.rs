use alloc::boxed::Box;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("No data to read")]
    NoData,
    #[error("Failed to read sample: {0:?}")]
    ReadFailure(Box<dyn core::error::Error>),
}
