use thiserror::Error;

use crate::errors::common::BufferAlreadyAllocated;

#[derive(Error, Debug)]
pub enum GraphAddError {
    #[error(transparent)]
    BufferAlreadyAllocated(#[from] BufferAlreadyAllocated),

    #[error(
        "port IDs must be densely packed starting from 0 within each direction; expected port ID {expected}, found {actual}"
    )]
    SparsePortIds { expected: usize, actual: usize },
}
