use crate::{
    Amplitude, AudioContextComputeError, AudioContextInterfaceData, AudioContextOptions,
    AudioContextUid, AudioPortAddress, ConnectionDescriptor, ConnectionError, Frame, NodeAudioData,
    NodeMessageData, NumChannels, Sample,
};

/// Underlying storage for the audio processes, audio graph, etc.
pub trait AudioContext<
    A: Amplitude,
    S: Sample<A>,
    F: Frame<A, S>,
    U: AudioContextUid,
    NAD: NodeAudioData<A, S, F, U>,
    NSD: NodeMessageData<A, S, F, U>,
    I: AudioContextInterfaceData<A, S, F, U, NAD, NSD>,
    N: NumChannels,
    P: AudioPortAddress,
    D: ConnectionDescriptor<N, U, P>,
>
{
    fn new_with_options(options: impl AudioContextOptions) -> Self;

    fn new() -> Self;

    fn compute_next_frame(&mut self) -> Result<Vec<I>, AudioContextComputeError>;

    fn compute_next_frame_with_data(
        &mut self,
        interface_data: impl Iterator<Item = impl AudioContextInterfaceData<A, S, F, U>>,
    ) -> Result<Vec<I>, AudioContextComputeError>;

    fn get_new_uid(&mut self) -> U;

    fn connect_ports(
        &mut self,
        this_port_address: impl AudioPortAddress,
        that_port_address: impl AudioPortAddress,
        num_channels: impl NumChannels,
    ) -> Result<&D, ConnectionError>;
}
