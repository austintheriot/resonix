use crate::{AudioPortAddress, NumChannels};

pub trait ConnectionDescriptor {
    fn num_channels(&self) -> impl NumChannels;

    /// Where the signal data is going to
    fn input_audio_port_address(&self) -> impl AudioPortAddress;

    /// Where the signal data is coming from
    fn output_audio_port_address(&self) -> impl AudioPortAddress;
}
