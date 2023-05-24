use audio::percentage::Percentage;

pub trait ChannelsAction {
    const DEFAULT_CHANNELS: f32;

    fn new(gain: impl Into<Percentage>) -> Self;

    fn get(&self) -> Percentage;

    fn set(&mut self, channels: impl Into<Percentage>);
}
