use super::channels_action::ChannelsAction;
use audio::{
    granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction, percentage::Percentage,
};

/// Wrapper around raw `f32` value for access on the audio thread
#[derive(Clone, Copy, Debug)]
pub struct Channels(Percentage);

impl ChannelsAction for Channels {
    const DEFAULT_CHANNELS: f32 = GranularSynthesizer::DEFAULT_CHANNELS;

    fn new(channels: impl Into<Percentage>) -> Self {
        Self(channels.into())
    }

    fn get(&self) -> Percentage {
        self.0
    }

    fn set(&mut self, channels: impl Into<Percentage>) {
        self.0 = channels.into();
    }
}

impl Default for Channels {
    fn default() -> Self {
        Self(Self::DEFAULT_CHANNELS.into())
    }
}
