use crate::AudioContext;

/// A uid that is unique to the current audio context
pub trait AudioContextUid {
    fn from_audio_context(audio_context: &mut impl AudioContext) -> Self;
}
