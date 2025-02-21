use std::marker::PhantomData;

use crate::{AudioContextUid, Frame, Sample};

pub struct NodeAudioData<
    A: resonix_types::Amplitude,
    S: resonix_types::Sample<A> = Sample<A>,
    F: resonix_types::Frame<A, S> = Frame<A, S>,
    U: resonix_types::AudioContextUid = AudioContextUid,
> {
    uid: U,
    audio_data: F,
    amplitude_type: PhantomData<A>,
    sample_type: PhantomData<S>,
}

impl<
        A: resonix_types::Amplitude,
        S: resonix_types::Sample<A>,
        F: resonix_types::Frame<A, S>,
        U: resonix_types::AudioContextUid,
    > resonix_types::NodeAudioData<A, S, F, U> for NodeAudioData<A, S, F, U>
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
