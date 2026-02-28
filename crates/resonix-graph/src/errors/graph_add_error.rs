use thiserror::Error;

use crate::errors::common::BufferAlreadyAllocated;

#[derive(Error, Debug)]
pub enum GraphAddError {
    #[error(transparent)]
    BufferAlreadyAllocated(#[from] BufferAlreadyAllocated),
}
