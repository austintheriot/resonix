pub struct AudioContextUid(pub(crate) usize);

impl AudioContextUid {
    pub(crate) fn new(uid: usize) -> Self {
        Self(uid)
    }
}

impl resonix_types::AudioContextUid for AudioContextUid {
    fn from_audio_context(
        audio_context: &mut impl resonix_types::AudioContext,
    ) -> impl resonix_types::AudioContextUid {
        audio_context.get_new_uid()
    }
}
