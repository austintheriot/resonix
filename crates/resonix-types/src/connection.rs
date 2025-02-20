use crate::{AudioContext, AudioPortAddress, ConnectionDescriptor, NumChannels};

pub trait Connection {
    fn new(
        audio_context: &mut impl AudioContext,
        this_port_address: impl AudioPortAddress,
        that_port_address: impl AudioPortAddress,
        num_channels: impl NumChannels,
    ) -> impl Connection;

    fn descriptor(&self) -> &impl ConnectionDescriptor;
}
