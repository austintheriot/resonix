use std::sync::Arc;

use crate::AudioPlayerContext;

/// Allows a function to pull whatever data it needs out of the audio Context
pub trait FromContext<D> {
    fn from_context(context: Arc<AudioPlayerContext<D>>) -> Self;
}
