use crate::AudioContextUid;

/// There are two necessary parts when informing an audio node
/// about what it is connected to: the node needs to know where the connection
/// is coming FROM and where the connection is going TO.
///
/// This serves as a unique per-audio-context address.
pub trait AudioPortAddress {
    fn audio_node_uid(&self) -> impl AudioContextUid;

    fn audio_port_uid(&self) -> impl AudioContextUid;
}
