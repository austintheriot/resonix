use crate::{NodeAudioData, NodeMessageData};

/// Vehicle for transmitting information to/from the AudioContext from the outside
/// world. Each piece of data is transmitted to its intended node by the audio context
/// once it is received.
///
/// This allows arbitrary outside systems to input audio data / effects data in whatever manner
/// they please and also get information back from the AudioContext in a system-agnostic way.
pub trait AudioContextInterfaceData {
    fn node_audio_data() -> impl Iterator<Item = impl NodeAudioData>;

    fn node_message_data() -> impl Iterator<Item = impl NodeMessageData>;
}
