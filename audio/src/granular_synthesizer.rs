use crate::grain::Grain;
use crate::granular_synthesizer_action::GranularSynthesizerAction;
use crate::max::Max;
use crate::min::Min;
use crate::percentage::Percentage;
use crate::{utils, IntSet};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

use std::sync::Arc;

/// Accepts a reference to a buffer of Vec<f32> audio sample data.
///
/// Generates random multi-channel audio grain output.
pub struct GranularSynthesizer {
    /// The maximum number of channels that can be generating samples via grains at a time.
    /// This can be used in conjunction with `density` to alter the number of playing grains
    /// during runtime.
    max_num_channels: u32,

    /// How many channels of grains to generate per frame (0.0 -> 1.0)
    /// A value of 1.0 corresponds to `max_num_channels` and a value of 0.o corresponds to no channels.
    density: Percentage,

    /// Sample rate of the surrounding context
    sample_rate: u32,

    /// External audio buffer that this GranularSynthesizer should read grains from
    buffer: Arc<Vec<f32>>,

    /// used to generate random indexes
    rng: StdRng,

    /// length as a percentage of the currently selected audio
    grain_len_min: Percentage,

    /// length as a percentage of the currently selected audio
    grain_len_max: Percentage,

    /// Samples that have been copied over from the audio buffer.
    /// This value gets multiplied with its corresponding `output_env_samples`
    /// to produce the final sample value.
    ///
    /// each array value = 1 channel
    output_buffer_samples: Vec<f32>,

    /// Envelope values that have been copied over to match the
    /// current progress of a grain.
    /// This value gets multiplied with its corresponding `output_buffer_samples`
    /// to produce the final sample value.
    ///
    /// each array value = 1 channel
    output_env_samples: Vec<f32>,

    /// The minimum index that samples can be taken from,
    /// ranging from 0.0 -> 1.0 (i.e. percentage of the buffer)
    selection_start: Percentage,

    /// The maximum index that samples can be taken from,
    /// ranging from 0.0 -> 1.0 (i.e. percentage of the buffer)
    selection_end: Percentage,

    /// This is a counter that gets incremented on every frame.
    /// This allows performing some actions (such as replacing grains) to occur
    /// only so often.
    refresh_counter: u32,

    /// This determines the interval (in samples) at which too-long grains are marked `finished`
    /// and `finished` grains are replaced with new ones.
    ///
    /// A higher interval produces a slower transition to new selected regions.
    ///
    /// A lower interval produces a faster transition to new selected regions.
    ///
    /// It is preferred for this interval to be a prime number to minimize the amount of
    /// sample overlap, where one grain is exactly in-sync with another, producing a unified
    /// and/or chorus effect (or exaggerated amplification).
    refresh_interval: u32,

    /// List of grains and their current progress through the buffer.
    ///
    /// 1 array element = 1 grain = 1 channel of audio
    fresh_grains: IntSet<Grain>,

    /// List of grains and their current progress through the buffer.
    ///
    /// 1 array element = 1 grain = 1 channel of audio
    finished_grains: IntSet<Grain>,
}

impl GranularSynthesizerAction for GranularSynthesizer {
    const DEFAULT_DENSITY: f32 = 0.5;

    fn new() -> Self {
        let default_buffer = Arc::new(Vec::new());

        let fresh_grains = IntSet::with_capacity(Self::DEFAULT_NUM_CHANNELS as usize);
        let mut finished_grains = IntSet::with_capacity(Self::DEFAULT_NUM_CHANNELS as usize);
        finished_grains.extend((0..Self::DEFAULT_NUM_CHANNELS).map(|i| {
            let mut new_grain = Self::new_grain();
            new_grain.uid = i;
            Some(new_grain)
        }));

        Self {
            sample_rate: Self::DEFAULT_SAMPLE_RATE,
            buffer: default_buffer,
            rng: rand::rngs::StdRng::from_entropy(),
            grain_len_min: Percentage::from(Self::GRAIN_LEN_MIN_MIN),
            grain_len_max: Percentage::from(Self::GRAIN_LEN_MAX_MIN),
            output_buffer_samples: vec![0.0; Self::DEFAULT_NUM_CHANNELS as usize],
            output_env_samples: vec![0.0; Self::DEFAULT_NUM_CHANNELS as usize],
            selection_start: Percentage::from(0.0),
            selection_end: Percentage::from(1.0),
            max_num_channels: Self::DEFAULT_NUM_CHANNELS,
            density: Percentage::from(Self::DEFAULT_DENSITY),
            refresh_counter: 0,
            refresh_interval: Self::DEFAULT_REFRESH_INTERVAL,
            fresh_grains,
            finished_grains,
        }
    }

    fn selection_start(&self) -> Percentage {
        self.selection_start
    }

    fn set_selection_start(&mut self, start: impl Into<Percentage>) -> &mut Self {
        self.selection_start = start.into();

        if self.selection_start > self.selection_end {
            // move end to "catch up" to the beginning
            self.set_selection_end(self.selection_start);
        }

        self
    }

    fn selection_end(&self) -> Percentage {
        self.selection_end
    }

    fn set_selection_end(&mut self, end: impl Into<Percentage>) -> &mut Self {
        self.selection_end = end.into();

        if self.selection_end < self.selection_start {
            // move beginning to be before the ending
            self.set_selection_start(self.selection_end);
        }

        self
    }

    fn set_grain_len_min(&mut self, grain_len_min: impl Into<Percentage>) -> &mut Self {
        self.grain_len_min = self.sanitize_grain_len_min(grain_len_min);

        // increase current grain length max to be greater than new min
        if self.grain_len_min > self.grain_len_max {
            self.set_grain_len_max(self.grain_len_min + Self::GRAIN_LEN_MIN_DIFFERENCE);
        }

        self
    }

    fn set_grain_len_max(&mut self, grain_len_max: impl Into<Percentage>) -> &mut Self {
        self.grain_len_max = self.sanitize_grain_len_max(grain_len_max);

        // decrease current grain length min to be less than the new max
        if self.grain_len_max < self.grain_len_min {
            self.set_grain_len_min(self.grain_len_max - Self::GRAIN_LEN_MIN_DIFFERENCE);
        }

        self
    }

    fn set_max_number_of_channels(&mut self, max_num_channels: u32) -> &mut Self {
        self.max_num_channels = max_num_channels;
        let max_num_channels = max_num_channels as usize;

        let prev_grain_length = self.finished_grains.len();
        // adjust grains to be as long as max number of channels
        if max_num_channels > prev_grain_length {
            self.finished_grains
                .extend((prev_grain_length..max_num_channels).map(|i| {
                    let mut new_grain = Self::new_grain();
                    new_grain.uid = i as u32;
                    Some(new_grain)
                }));
            self.fresh_grains
                .extend((prev_grain_length..max_num_channels).map(|_| None));
        } else if max_num_channels < prev_grain_length {
            self.fresh_grains.truncate(max_num_channels);
            self.finished_grains.truncate(max_num_channels);
        }

        // adjust samples buffer to be as long as max number of channels
        if max_num_channels > self.output_buffer_samples.len() {
            let num_extra_samples = max_num_channels - self.output_buffer_samples.len();
            self.output_buffer_samples
                .extend(vec![0.0; num_extra_samples]);
        } else if max_num_channels < self.output_buffer_samples.len() {
            self.output_buffer_samples.truncate(max_num_channels);
        }

        // adjust envelope buffer to be as long as max number of channels
        if max_num_channels > self.output_env_samples.len() {
            let num_extra_samples = max_num_channels - self.output_env_samples.len();
            self.output_env_samples.extend(vec![0.0; num_extra_samples]);
        } else if max_num_channels < self.output_env_samples.len() {
            self.output_env_samples.truncate(max_num_channels);
        }

        self
    }

    fn set_density(&mut self, density: impl Into<Percentage>) -> &mut Self {
        self.density = density.into();
        self
    }

    fn density(&self) -> Percentage {
        self.density
    }

    fn set_buffer(&mut self, buffer: Arc<Vec<f32>>) -> &mut Self {
        let buffer_len_samples = buffer.len();
        self.buffer = buffer;

        // find any grains that extend past the current buffer length
        let finished_grain_indexes: Vec<_> = self
            .fresh_grains
            .iter()
            .enumerate()
            .filter(|(_, grain)| {
                grain.end_frame > buffer_len_samples
                    || grain.start_frame > buffer_len_samples
                    || grain.len > buffer_len_samples
                    || grain.current_frame > buffer_len_samples
            })
            .map(|(i, _)| i)
            .collect();

        // move grains into the finished list
        finished_grain_indexes
            .iter()
            .filter_map(|&i| self.fresh_grains.remove(i))
            .for_each(|mut removed_grain| {
                removed_grain.finished = true;
                self.finished_grains.insert(removed_grain);
            });

        self
    }

    fn refresh_interval(&self) -> u32 {
        self.refresh_interval
    }

    fn set_refresh_interval(&mut self, refresh_interval: u32) -> &mut Self {
        self.refresh_interval = Self::sanitize_refresh_interval(refresh_interval);

        self
    }

    fn next_frame_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut Vec<f32>,
    ) -> &'a mut Vec<f32> {
        // by only filtering/refreshing grains at an interval, it blends one sound into the other
        // decrease speed of refreshes to blend sounds together
        if self.refresh_counter % self.refresh_interval() == 0 {
            self.filter_one_long_grain();
            self.refresh_grain();
        }

        self.increment_refresh_counter();

        self.fill_buffer_and_env_samples();
        self.frame_data(frame_data_buffer)
    }

    fn next_frame(&mut self) -> Vec<f32> {
        let mut frame_data_buffer = vec![0.0; self.num_channels_for_frame()];
        self.next_frame_into_buffer(&mut frame_data_buffer);
        frame_data_buffer
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = sample_rate;

        self
    }

    fn grain_len_min(&self) -> Percentage {
        self.grain_len_min
    }

    fn grain_len_max(&self) -> Percentage {
        self.grain_len_max
    }
}

// internal logic to support public GranularSynthesizer interface
impl GranularSynthesizer {
    fn sanitize_grain_len_min(&self, grain_len_min: impl Into<Percentage>) -> Percentage {
        grain_len_min
            .into()
            // should be smaller than existing max
            .min((self.grain_len_max - Self::GRAIN_LEN_MIN_DIFFERENCE).into())
            // should be <= largest possible length
            .min(Self::GRAIN_LEN_MIN_MAX.into())
            // should be >= smallest possible length
            .max(Self::GRAIN_LEN_MIN_MIN.into())
    }

    fn sanitize_grain_len_max(&self, grain_len_max: impl Into<Percentage>) -> Percentage {
        grain_len_max
            .into()
            // should be bigger than existing min
            .max((self.grain_len_min + Self::GRAIN_LEN_MIN_DIFFERENCE).into())
            // should be >= smallest possible length
            .max(Self::GRAIN_LEN_MAX_MIN.into())
            // should be <= largest possible length
            .min(Self::GRAIN_LEN_MAX_MAX.into())
    }

    fn grain_len_min_in_samples(&self) -> u32 {
        let selection_len_in_samples = self.selection_len_in_samples() as f32;
        let grain_len_min_in_samples = selection_len_in_samples * self.grain_len_min;
        grain_len_min_in_samples as u32
    }

    fn grain_len_max_in_samples(&self) -> u32 {
        let selection_len_in_samples = self.selection_len_in_samples() as f32;
        let grain_len_max_in_samples = selection_len_in_samples * self.grain_len_max;
        grain_len_max_in_samples as u32
    }

    /// Iterates through array of grains (1 grain for each channel), and refreshes 1
    /// grain that was previously finished with a new range of buffer indexes.
    fn refresh_grain(&mut self) {
        // get start and end of selection
        let selection_start_index = self.selection_start_in_samples();
        let selection_end_index = self.selection_end_in_samples();
        let selection_len_in_samples = self.selection_len_in_samples();

        let grain_len_max_samples = self.grain_len_max_in_samples();
        let grain_len_min_samples = self.grain_len_min_in_samples();

        // get largest possible grain length:
        // the smaller value between (selection_end - selection_start) & grain_len_max]
        let largest_possible_grain_len = selection_len_in_samples.min(grain_len_max_samples);

        // get smallest possible grain length:
        // the smaller value between (selection_end - selection_start) & grain_len_min
        let smallest_possible_grain_len = selection_len_in_samples.min(grain_len_min_samples);

        // if nothing is selected, there's no use in refreshing grains with empty data
        let selection_is_empty = selection_start_index >= selection_end_index;
        let grain_len_range_is_small =
            largest_possible_grain_len - smallest_possible_grain_len <= 20;

        if selection_is_empty {
            return;
        }

        if let Some(finished_grain) = self.finished_grains.pop_first() {
            // get random length
            let mut grain_len = if grain_len_range_is_small {
                // there are fewer errors that can happen with larger grains
                // (less divide by 0 errors, etc.)
                largest_possible_grain_len
            } else {
                self.rng
                    .gen_range(smallest_possible_grain_len..=largest_possible_grain_len)
            };

            let largest_start_index = selection_end_index - grain_len;

            // if the largest possible start index and the actual start index are very close,
            // then just use the start index (prevents silence when min & max are both at 1.0)
            let start_index_range_is_close = (largest_start_index - selection_start_index) < 20;

            // if the start is close, then that means the the grain should just play the whole buffer
            if start_index_range_is_close {
                grain_len = selection_end_index;
            }

            let grain_start_index = if start_index_range_is_close {
                selection_start_index
            } else {
                // get random index inside selection
                self.rng
                    .gen_range(selection_start_index..=largest_start_index)
            };

            let grain_end_index = grain_start_index + grain_len;

            let fresh_grain = Grain::new(
                grain_start_index as usize,
                grain_end_index as usize,
                // keep the same uid as previous grain
                finished_grain.uid,
            );
            self.fresh_grains.insert(fresh_grain);
        }
    }

    fn selection_start_in_samples(&self) -> u32 {
        ((self.buffer.len() as f32 * self.selection_start) as u32)
            .max(0)
            .min(self.buffer.len() as u32)
    }

    fn selection_end_in_samples(&self) -> u32 {
        ((self.buffer.len() as f32 * self.selection_end) as u32)
            .max(0)
            .min(self.buffer.len() as u32)
    }

    fn selection_len_in_samples(&self) -> u32 {
        (self.selection_end_in_samples() - self.selection_start_in_samples()).max(0)
    }

    /// Prevent long grains from lingering when max length and/or selection has changed
    ///
    /// Finds a single grain that exceeds the current selection length and marks it as finished
    /// If no grain is found that exceeds the current selection length, no other action is taken.
    fn filter_one_long_grain(&mut self) {
        let grain_len_max_in_samples = self.grain_len_max_in_samples() as usize;
        let selection_len_in_samples = self.selection_len_in_samples() as usize;

        // find all grains that are too long
        let long_grain_index = self
            .fresh_grains
            .iter()
            .enumerate()
            .find(|(_, grain)| {
                let remaining_grain_samples = grain.remaining_samples();
                remaining_grain_samples > grain_len_max_in_samples
                    || remaining_grain_samples > selection_len_in_samples
            })
            .map(|(i, _)| i);

        // move the grains into the finished_grains list
        if let Some(i) = long_grain_index {
            self.fresh_grains.remove(i).map(|mut removed_grain| {
                removed_grain.finished = true;
                self.finished_grains.insert(removed_grain);
            });
        }
    }

    /// Fills in buffer & envelope sample data for each channel
    /// based on the current state of the grain for each channel.
    ///
    /// Each buffer sample and envelope sample must be coordinated/aligned to prevent
    /// audio clipping and/or unexpected audio results.
    fn fill_buffer_and_env_samples(&mut self) {
        // get value of each grain's current index in the buffer for each channel
        self.fresh_grains
            .iter_mut()
            .enumerate()
            .for_each(|(i, grain)| {
                // this can happen if a grain has finished, but the selection
                // size is 0, so the grain can't get refreshed with more data
                if grain.finished {
                    self.output_buffer_samples[i] = 0.0;
                    self.output_env_samples[i] = 0.0;
                    return;
                }

                let grain_len = grain.len.max(1) as f32;
                let envelope_percent =
                    ((grain.current_frame - grain.start_frame) as f32) / grain_len;
                let envelope_value =
                    utils::generate_triangle_envelope_value_from_percent(envelope_percent);
                let sample_value = self.buffer[grain.current_frame];

                self.output_buffer_samples[i] = sample_value;
                self.output_env_samples[i] = envelope_value;

                grain.next_frame();
            });
    }

    /// this represents the number of channels actually in use
    fn num_channels_for_frame(&self) -> usize {
        (self.max_num_channels as f32 * self.density) as usize
    }

    /// Combines current buffer and envelope sample values to calculate a full audio frame
    /// (where each channel gets a single audio output value).
    fn frame_data<'a>(&self, frame_data_buffer: &'a mut Vec<f32>) -> &'a mut Vec<f32> {
        let num_channels_for_frame = self.num_channels_for_frame();
        frame_data_buffer.resize(num_channels_for_frame, 0.0);
        for (i, channel) in frame_data_buffer.iter_mut().enumerate() {
            let buffer_sample = self
                .output_buffer_samples
                .get(i)
                .copied()
                .expect("output_buffer_samples length should match max number of grains");

            let envelope_sample = self
                .output_env_samples
                .get(i)
                .copied()
                .expect("output_env_samples length should match max number of grains");

            // if these buffers have not been filled up yet, just return 0.0
            *channel = buffer_sample * envelope_sample;
        }

        frame_data_buffer
    }

    fn increment_refresh_counter(&mut self) {
        self.refresh_counter = self.refresh_counter.wrapping_add(1);
    }
}
