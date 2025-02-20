use crate::{AudioContextUid, AudioPortAddress, NumChannels};

pub trait ConnectionDescriptor {
    fn audio_connection_uid(&self) -> &impl AudioContextUid;

    fn num_channels(&self) -> &impl NumChannels;

    /// Where the signal data is going to
    fn input_audio_port_address(&self) -> &impl AudioPortAddress;

    /// Where the signal data is coming from
    fn output_audio_port_address(&self) -> &impl AudioPortAddress;
}
