use resonix_types::NodeAudioData;

use crate::NodeMessageData;

pub struct AudioContextInterfaceData {}

impl<
        A: resonix_types::Amplitude,
        S: resonix_types::Sample<A>,
        F: resonix_types::Frame<A, S>,
        U: resonix_types::AudioContextUid,
    > resonix_types::AudioContextInterfaceData<A, S, F, U> for AudioContextInterfaceData
{
    fn node_audio_data() -> impl Iterator<Item = impl resonix_types::NodeAudioData<A, S, F, U>> {
        todo!()
    }

    fn node_message_data() -> impl Iterator<Item = impl resonix_types::NodeMessageData<A, S, F, U>>
    {
        std::iter::empty::<NodeMessageData>()
    }
}
