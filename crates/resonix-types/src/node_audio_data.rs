use crate::{Amplitude, AudioContextUid, Frame, Sample};

pub trait NodeAudioData<A: Amplitude, S: Sample<A>, F: Frame<A, S>, U: AudioContextUid> {
    fn node_uid(&self) -> &U;

    fn audio_data(&self) -> &F;

    fn into_audio_data(self) -> F;
}
