use resonix_types::Frame;

use crate::AudioContextUid;

pub struct NodeAudioData<
    A: resonix_types::Amplitude,
    S: resonix_types::Sample = Sample<Amplitude>,
    F: resonix_types::Frame = Frame,
    U: resonix_types::AudioContextUid = AudioContextUid,
> {
    uid: U,
    audio_data: F,
}

impl<
        A: resonix_types::Amplitude,
        S: resonix_types::Sample,
        F: resonix_types::Framee,
        U: resonix_types::AudioContextUid,
    > resonix_types::NodeAudioData for NodeAudioData<A, S, F, U>
{
    fn node_uid(&self) -> &U {
        &self.uid
    }

    fn audio_data(&self) -> &F {
        &self.audio_data
    }

    fn into_audio_data(self) -> F {
        self.audio_data
    }
}
