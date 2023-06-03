use std::sync::Arc;

use crate::AudioPlayerContext;

/// Allows a function to pull whatever data it needs out of the audio player context
/// and whatever user-specified data is specified inside
pub trait UserDataFromContext<D> {
    fn from_context(context: Arc<AudioPlayerContext<D>>) -> Self;
}
