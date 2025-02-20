use crate::{AudioContextUid, ConnectionDescriptor};

pub trait AudioNode {
    fn audio_note_uid(&self) -> impl AudioContextUid;

    fn input_connection_descriptors(&self) -> Option<&[impl ConnectionDescriptor]>;

    fn output_connection_descriptors(&self) -> Option<&[impl ConnectionDescriptor]>;
}
