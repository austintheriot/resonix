use crate::AudioPlayerContext;

/// Allows a function to pull whatever data it needs out of the audio Context
pub trait FromContext<'a, 'c: 'a, D> {
    // borrow of the
    fn from_context(context: &'c AudioPlayerContext<D>) -> Self;
}
