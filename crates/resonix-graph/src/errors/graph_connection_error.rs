use thiserror::Error;

use crate::errors::common::BufferAlreadyAllocated;

#[derive(Error, Debug)]
pub enum GraphConnectionError {
    #[error(transparent)]
    BufferAlreadyAllocated(#[from] BufferAlreadyAllocated),
}
