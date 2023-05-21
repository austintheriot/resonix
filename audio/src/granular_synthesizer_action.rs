use std::sync::Arc;

use crate::{grain::Grain, percentage::Percentage};

/// Public interface to the GranularSynthesizer.
///
/// This interface is extracted into a constant so that a
/// GranularSynthesizerHandle can more easily re-export this logic.
pub trait GranularSynthesizerAction {
    const DENSITY_MAX: f32 = 1.0;

    const DENSITY_MIN: f32 = 0.0;

    const DEFAULT_NUM_CHANNELS: u32 = 2;

    const DEFAULT_DENSITY: f32 = 1.0;

    const GRAIN_LEN_MIN_DIFFERENCE: f32 = 0.05;

    /// the smallest possible length of grain, given as a percentage of the currently selected audio
    const GRAIN_LEN_MIN_MIN: f32 = 0.0;

    const GRAIN_LEN_MIN_MAX: f32 = Self::GRAIN_LEN_MAX_MAX - Self::GRAIN_LEN_MIN_DIFFERENCE;

    const GRAIN_LEN_MAX_MIN: f32 = Self::GRAIN_LEN_MIN_MIN + Self::GRAIN_LEN_MIN_DIFFERENCE;

    /// the largest possible length of grain, given as a percentage of the currently selected audio
    const GRAIN_LEN_MAX_MAX: f32 = 1.0;

    const DEFAULT_SAMPLE_RATE: u32 = 44100;

    /// This is the sample interval at which grains are filtered / refreshed.
    /// Using a prime number leads to the least periodic overlap in grains.
    const DEFAULT_REFRESH_INTERVAL: u32 = 271;

    const REFRESH_INTERVAL_MIN: u32 = 17;

    const REFRESH_INTERVAL_MAX: u32 = 1009;

    /// Creates a new GranularSynthesizer instance
    fn new() -> Self;

    fn set_selection_start(&mut self, start: impl Into<Percentage>) -> &mut Self;

    fn selection_start(&self) -> Percentage;

    fn selection_end(&self) -> Percentage;

    fn set_selection_end(&mut self, start: impl Into<Percentage>) -> &mut Self;

    fn set_grain_len_min(&mut self, input_min_len_in_ms: impl Into<Percentage>) -> &mut Self;

    fn set_grain_len_max(&mut self, input_max_len_in_ms: impl Into<Percentage>) -> &mut Self;

    fn set_max_number_of_channels(&mut self, max_num_channels: u32) -> &mut Self;

    fn set_density(&mut self, density: impl Into<Percentage>) -> &mut Self;

    fn density(&self) -> Percentage;

    fn sanitize_refresh_interval(refresh_interval: u32) -> u32 {
        refresh_interval
            .max(Self::REFRESH_INTERVAL_MIN)
            .min(Self::REFRESH_INTERVAL_MAX)
    }

    fn refresh_interval(&self) -> u32;

    fn set_refresh_interval(&mut self, refresh_interval: u32) -> &mut Self;

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
    /// where each channel gets its own, independent value
    /// based on the progression of that audio channel's grain.
    /// 
    /// Reads data directly into a pre-existing buffer, resizing it 
    /// to match the number of audio channels in this frame if the
    /// number of channels does not match the length of the vector.
    fn next_frame_into_buffer<'a>(&mut self, frame_data_buffer: &'a mut Vec<f32>) -> &'a mut Vec<f32>;

    /// Returns a full audio frame (1 array element = 1 audio channel value),
    /// where each channel gets its own, independent value
    /// based on the progression of that audio channel's grain.
    /// 
    /// Returns a newly allocated buffer to hold the frame data.
    fn next_frame(&mut self) -> Vec<f32>;

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
            uid: 0,
        }
    }

    fn grain_len_min(&self) -> Percentage;

    fn grain_len_max(&self) -> Percentage;
}
