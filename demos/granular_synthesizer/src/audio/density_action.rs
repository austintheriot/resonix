use audio::percentage::Percentage;

pub trait DensityAction {
    const DEFAULT_DENSITY: f32;

    fn new(gain: impl Into<Percentage>) -> Self;

    fn get(&self) -> Percentage;

    fn set(&mut self, density: impl Into<Percentage>);
}
