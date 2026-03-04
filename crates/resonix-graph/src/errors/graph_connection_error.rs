use thiserror::Error;

use crate::errors::common::BufferAlreadyAllocated;

#[derive(Error, Debug)]
pub enum GraphConnectionError {
    #[error(transparent)]
    BufferAlreadyAllocated(#[from] BufferAlreadyAllocated),

    #[error(
        "channel count mismatch: output port has {start_channels} channel(s), input port has {end_channels}"
    )]
    ChannelCountMismatch {
        start_channels: usize,
        end_channels: usize,
    },
}
