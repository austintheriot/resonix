use thiserror::Error;

#[derive(Debug, Error)]
#[error("Internal error: buffer already allocated")]
pub struct BufferAlreadyAllocated;
