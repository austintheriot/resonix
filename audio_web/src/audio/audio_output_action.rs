pub trait AudioOutputAction {
    const NUM_FRAMES_DEFAULT: usize = 20;
    const NUM_CHANNELS_DEFAULT: usize = 2;

    fn add_frame(&mut self, frame: Vec<f32>);

    fn get_simple_moving_average(&self) -> Vec<f32>;
}
