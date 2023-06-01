use crate::AudioPlayerContext;

/// Allows a function to pull whatever data it needs out of the audio Context
pub trait FromContext {
    fn from_context(context: &AudioPlayerContext) -> Self;
}
