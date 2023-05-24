use audio::NumChannels;

pub trait NumChannelsAction {
    const DEFAULT_CHANNELS: usize;

    fn new(gain: impl Into<usize>) -> Self;

    fn get(&self) -> NumChannels;

    fn set(&mut self, channels: impl Into<usize>);
}
