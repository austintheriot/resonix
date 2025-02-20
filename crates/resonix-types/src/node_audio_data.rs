use crate::{AudioContextUid, Frame};

pub trait NodeAudioData {
    fn node_uid(&self) -> impl AudioContextUid;

    fn audio_data(&self) -> &impl Frame;

    fn into_audio_data(self) -> impl Frame;
}
