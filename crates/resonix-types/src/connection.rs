use crate::{
    Amplitude, AudioContext, AudioContextInterfaceData, AudioContextUid, AudioPortAddress,
    ConnectionDescriptor, Frame, NumChannels, Sample,
};

pub trait Connection<
    A: Amplitude,
    S: Sample<A>,
    F: Frame<A, S>,
    U: AudioContextUid,
    I: AudioContextInterfaceData<A, S, F, U>,
    N: NumChannels,
    P: AudioPortAddress,
    D: ConnectionDescriptor<N, U, P>,
>
{
    fn new(
        audio_context: &mut impl AudioContext<A, S, F, U, I, N, P, D>,
        this_port_address: impl AudioPortAddress,
        that_port_address: impl AudioPortAddress,
        num_channels: impl NumChannels,
    ) -> Self;

    fn descriptor(&self) -> &impl ConnectionDescriptor<N, U, P>;
}
