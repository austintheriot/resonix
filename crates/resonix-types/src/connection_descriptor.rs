use crate::{AudioContextUid, AudioPortAddress, NumChannels};

pub trait ConnectionDescriptor<N: NumChannels, U: AudioContextUid, P: AudioPortAddress> {
    fn audio_connection_uid(&self) -> &U;

    fn num_channels(&self) -> &N;

    /// Where the signal data is going to
    fn input_audio_port_address(&self) -> &P;

    /// Where the signal data is coming from
    fn output_audio_port_address(&self) -> &P;
}
