use crate::{Amplitude, AudioContextUid, Frame, NodeAudioData, NodeMessageData, Sample};

/// Vehicle for transmitting information to/from the AudioContext from the outside
/// world. Each piece of data is transmitted to its intended node by the audio context
/// once it is received.
///
/// This allows arbitrary outside systems to input audio data / effects data in whatever manner
/// they please and also get information back from the AudioContext in a system-agnostic way.
pub trait AudioContextInterfaceData<
    A: Amplitude,
    S: Sample<A>,
    F: Frame<A, S>,
    U: AudioContextUid,
    NAD: NodeAudioData<A, S, F, U>,
    NSD: NodeMessageData<A, S, F, U>,
>
{
    fn node_audio_data() -> Vec<NAD>;

    fn node_message_data() -> Vec<NSD>;
}
