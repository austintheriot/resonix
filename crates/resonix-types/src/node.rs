use crate::{
    Amplitude, AudioContext, AudioContextInterfaceData, AudioContextUid, AudioPortAddress,
    ConnectionDescriptor, Frame, NodeDescriptor, NumChannels, Sample,
};

pub trait Node<
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
    type Options;

    fn new(audio_context: &mut impl AudioContext<A, S, F, U, I, N, P, D>) -> Self;

    fn new_with_options(
        audio_context: &mut impl AudioContext<A, S, F, U, I, N, P, D>,
        options: Self::Options,
    ) -> Self;

    fn descriptor(&self) -> &impl NodeDescriptor;

    fn current_input_connections(
        &self,
    ) -> Option<&impl Iterator<Item = impl ConnectionDescriptor<N, U, P>>>;

    fn current_output_connections(
        &self,
    ) -> Option<&impl Iterator<Item = impl ConnectionDescriptor<N, U, P>>>;
}
