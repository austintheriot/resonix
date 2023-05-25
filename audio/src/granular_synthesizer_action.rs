use std::{sync::Arc, time::Duration};

use crate::{grain::Grain, percentage::Percentage, NumChannels, EnvelopeType};

/// Public interface to the GranularSynthesizer.
///
/// This interface is extracted into a constant so that a
/// GranularSynthesizerHandle can more easily re-export this logic.
pub trait GranularSynthesizerAction {
    const CHANNELS_MAX: f32 = 1.0;

    const CHANNELS_MIN: f32 = 0.0;

    const DEFAULT_NUM_CHANNELS: usize = 2;

    const GRAIN_LEN_MIN: Duration = Duration::from_millis(20);

    const GRAIN_LEN_MAX: Duration = Duration::from_millis(1000);

    const DEFAULT_GRAIN_LEN: Duration = Duration::from_millis(20);

    const GRAIN_INITIALIZATION_DELAY_MIN: Duration = Duration::ZERO;

    const GRAIN_INITIALIZATION_DELAY_MAX: Duration = Duration::from_millis(1000);

    // using a prime number helps prevent grains whose playback start/stop
    // times overlap with one another
    const DEFAULT_GRAIN_INITIALIZATION_DELAY: Duration = Duration::from_millis(17);

    const DEFAULT_SAMPLE_RATE: u32 = 44100;

    /// This is the sample interval at which grains are filtered / refreshed.
    /// Using a prime number leads to the least periodic overlap in grains.
    const DEFAULT_REFRESH_INTERVAL: u32 = 271;

    /// Creates a new GranularSynthesizer instance
    fn new() -> Self;

    fn set_selection_start(&mut self, start: impl Into<Percentage>) -> &mut Self;

    fn selection_start(&self) -> Percentage;

    fn selection_end(&self) -> Percentage;

    fn grain_initialization_delay(&self) -> Duration;

    fn set_selection_end(&mut self, start: impl Into<Percentage>) -> &mut Self;

    fn set_grain_len(&mut self, input_min_len_in_ms: impl Into<Duration>) -> &mut Self;

    fn set_num_channels(&mut self, channels: impl Into<NumChannels>) -> &mut Self;

    fn set_grain_initialization_delay(&mut self, delay: impl Into<Duration>) -> &mut Self;

    fn num_channels(&self) -> NumChannels;

    fn sanitize_grain_initialization_delay(delay: Duration) -> Duration {
        delay
            .max(Self::GRAIN_INITIALIZATION_DELAY_MIN)
            .min(Self::GRAIN_INITIALIZATION_DELAY_MAX)
    }

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
    fn next_frame_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut Vec<f32>,
    ) -> &'a mut Vec<f32>;

    /// Returns a full audio frame (1 array element = 1 audio channel value),
    /// where each channel gets its own, independent value
    /// based on the progression of that audio channel's grain.
    ///
    /// Returns a newly allocated buffer to hold the frame data.
    fn next_frame(&mut self) -> Vec<f32>;

    /// This should be set BEFORE calling `set_grain_len_min` or `set_grain_len_max`
    fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self;

    /// Produces an uninitialized grain for filling the initial array of Grains
    ///
    /// Once it is time to actually produce an audio sample from the buffer,
    /// each grain will be initialized with a random start/end index, etc.
    fn new_grain(uid: u32) -> Grain {
        Grain {
            current_frame: 0,
            end_frame: 0,
            finished: true,
            len: 0,
            start_frame: 0,
            uid,
            init: false,
        }
    }

    /// Volume envelope that determines the volume of the grain as it plays through
    fn set_envelope(&mut self, envelope_type: EnvelopeType) -> &mut Self;

    fn grain_len(&self) -> Duration;
}
