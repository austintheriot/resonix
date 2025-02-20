use crate::{
    AudioContextComputeError, AudioContextInterfaceData, AudioContextOptions, AudioContextUid,
    AudioPortAddress, ConnectionDescriptor, ConnectionError, NumChannels,
};

/// Underlying storage for the audio processes, audio graph, etc.
pub trait AudioContext {
    fn new_with_options(options: impl AudioContextOptions) -> impl AudioContext;

    fn new() -> impl AudioContext;

    fn compute_next_frame(
        &mut self,
    ) -> Result<impl Iterator<Item = impl AudioContextInterfaceData>, AudioContextComputeError>;

    fn compute_next_frame_with_data(
        &mut self,
        interface_data: impl Iterator<Item = impl AudioContextInterfaceData>,
    ) -> Result<impl Iterator<Item = impl AudioContextInterfaceData>, AudioContextComputeError>;

    fn get_new_uid(&mut self) -> impl AudioContextUid;

    fn connect_ports(
        &mut self,
        this_port_address: impl AudioPortAddress,
        that_port_address: impl AudioPortAddress,
        num_channels: impl NumChannels,
    ) -> Result<&impl ConnectionDescriptor, ConnectionError>;
}
