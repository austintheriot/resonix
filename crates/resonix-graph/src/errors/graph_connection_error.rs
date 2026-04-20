use thiserror_no_std::Error;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn channel_count_mismatch_display_shows_both_channel_counts() {
        let error = GraphConnectionError::ChannelCountMismatch {
            start_channels: 1,
            end_channels: 2,
        };
        assert_eq!(
            error.to_string(),
            "channel count mismatch: output port has 1 channel(s), input port has 2"
        );
    }

    #[test]
    fn from_buffer_already_allocated_wraps_into_correct_variant() {
        let error: GraphConnectionError = BufferAlreadyAllocated.into();
        assert!(matches!(
            error,
            GraphConnectionError::BufferAlreadyAllocated(_)
        ));
    }

    #[test]
    fn buffer_already_allocated_display_is_transparent() {
        let error: GraphConnectionError = BufferAlreadyAllocated.into();
        assert_eq!(
            error.to_string(),
            "Internal error: buffer already allocated"
        );
    }
}
