use resonix_types::NodeAudioData;

use crate::NodeMessageData;

pub struct AudioContextInterfaceData {}

impl resonix_types::AudioContextInterfaceData for AudioContextInterfaceData {
    fn node_audio_data() -> impl Iterator<Item = impl resonix_types::NodeAudioData> {
        vec![].into_iter()
    }

    fn node_message_data() -> impl Iterator<Item = impl resonix_types::NodeMessageData> {
        std::iter::empty::<NodeMessageData>()
    }
}
