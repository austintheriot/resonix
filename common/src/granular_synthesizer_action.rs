use std::sync::Arc;

use crate::grain::Grain;

/// Public interface to the GranularSynesizer.
///
/// This interface is extracted into a constant so that a
/// GranularSynthesizerHandle can more easily re-export this logic.
pub trait GranularSynthesizerAction {
    const DENSITY_MAX: f32 = 1.0;

    const DENSITY_MIN: f32 = 0.0;

    const DEFAULT_NUM_CHANNELS: u32 = 2;

    const DEFAULT_DENSITY: f32 = 1.0;

    /// the smallest possible length of grain, given in samples
    const GRAIN_LEN_ABSOLUTE_MIN_IN_MS: u32 = 1;

    /// Creates a new GranularSynthesizer instance
    fn new(buffer: Arc<Vec<f32>>, sample_rate: u32) -> Self;

    fn set_selection_start(&mut self, start: f32) -> &mut Self;

    fn set_selection_end(&mut self, start: f32) -> &mut Self;

    /// The smallest possible grain length is 1 ms,
    /// and the min grain length and can never exceed the max.
    fn set_grain_len_min(&mut self, input_min_len_in_ms: u32) -> &mut Self;

    /// The maximum grain length is the size of the buffer itself,
    /// and max grain length can never be lower than the min width (or 1ms--whichever is higher)
    fn set_grain_len_max(&mut self, input_max_len_in_ms: u32) -> &mut Self;

    fn set_max_number_of_channels(&mut self, max_num_channels: u32) -> &mut Self;

    fn set_density(&mut self, density: f32) -> &mut Self;

    /// Replace the internal buffer reference with a different one.
    ///
    /// Any existing / currently playing grains that extend past the new buffer
    /// will be replaced with new ones on the next call to `next_frame`.
    ///
    /// Any existing / current playing grains that are compatible with new buffer
    /// length will keep their internal state unchanged and will sample from the
    /// new buffer on the next frame.
    fn set_buffer(&mut self, buffer: Arc<Vec<f32>>) -> &mut Self;

    /// Returns a full audio frame (1 array element = 1 audio channel value),
    /// where each channel gets its own, indepedent value
    /// based on the progression of that audio channel's grain.
    fn next_frame(&mut self) -> Vec<f32>;

    /// Returns a density value within an acceptable range
    fn sanitize_density(density: f32) -> f32 {
        density.max(Self::DENSITY_MIN).min(Self::DENSITY_MAX)
    }

    /// This should be set BEFORE calling `set_grain_len_min` or `set_grain_len_max`
    fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self;

    /// Produces an unitialized grain for filling the initial array of Grains
    ///
    /// Once it is time to actually produce an audio sample from the buffer,
    /// each grain will be initialized with a randaom start/end index, etc.
    fn new_grain() -> Grain {
        Grain {
            current_frame: 0,
            end_frame: 0,
            finished: true,
            len: 0,
            start_frame: 0,
        }
    }
}
