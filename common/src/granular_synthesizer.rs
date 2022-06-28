use crate::grain::Grain;
use crate::granular_synthesizer_action::GranularSynthesizerAction;
use crate::utils;
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
    density: f32,

    /// Sample rate of the surrounding context
    sample_rate: u32,

    /// External audio buffer that this GranularSynthesizer should read grains from
    buffer: Arc<Vec<f32>>,

    /// List of grains and their current progress through the buffer.
    ///
    /// 1 array element = 1 grain = 1 channel of audio
    grains: Vec<Grain>,

    /// used to generate random indexes
    rng: StdRng,

    /// length in milliseconds
    grain_len_min: u32,

    /// original user input
    grain_len_min_raw: u32,

    /// length in milliseconds
    grain_len_max: u32,

    // original user input
    grain_len_max_raw: u32,

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
    selection_start: f32,

    /// The maximum index that samples can be taken from,
    /// ranging from 0.0 -> 1.0 (i.e. percentage of the buffer)
    selection_end: f32,
}

impl GranularSynthesizerAction for GranularSynthesizer {
    fn new() -> Self {
        let default_buffer = Arc::new(Vec::new());
        let grain_len_min_default = Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS;
        let grain_len_max_deafult =
            Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS + Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS;

        Self {
            sample_rate: Self::DEFAULT_SAMPLE_RATE,
            buffer: default_buffer,
            grains: vec![Self::new_grain(); Self::DEFAULT_NUM_CHANNELS as usize],
            rng: rand::rngs::StdRng::from_entropy(),
            grain_len_min: grain_len_min_default,
            grain_len_min_raw: grain_len_min_default,
            grain_len_max: grain_len_max_deafult,
            grain_len_max_raw: grain_len_max_deafult,
            output_buffer_samples: vec![0.0; Self::DEFAULT_NUM_CHANNELS as usize],
            output_env_samples: vec![0.0; Self::DEFAULT_NUM_CHANNELS as usize],
            selection_start: 0.0,
            selection_end: 1.0,
            max_num_channels: Self::DEFAULT_NUM_CHANNELS,
            density: Self::DEFAULT_DENSITY,
        }
    }

    fn set_selection_start(&mut self, start: f32) -> &mut Self {
        self.selection_start = Self::sanitize_selection(start);

        if self.selection_start > self.selection_end {
            // move end to "catch up" to the beginning
            self.set_selection_end(self.selection_start);
        }

        // prevent long grains from lingering when selection size has changed
        self.filter_long_grains();

        self
    }

    fn set_selection_end(&mut self, end: f32) -> &mut Self {
        self.selection_end = Self::sanitize_selection(end);

        if self.selection_end < self.selection_start {
            // move beginning to be before the ending
            self.set_selection_start(self.selection_end);
        }

        // prevent long grains from lingering when selection size has changed
        self.filter_long_grains();

        self
    }

    fn set_grain_len_min(&mut self, grain_len_min_in_ms: u32) -> &mut Self {
        self.grain_len_min_raw = grain_len_min_in_ms;
        self.grain_len_min = self.sanitize_grain_len_min(grain_len_min_in_ms as u32);

        // increase current grain length max to be greater than new min
        if self.grain_len_min > self.grain_len_max {
            self.set_grain_len_max(
                self.grain_len_min + GranularSynthesizer::GRAIN_LEN_ABSOLUTE_MIN_IN_MS,
            );
        }

        // prevent long grains from lingering when grain length has changed
        self.filter_long_grains();

        self
    }

    fn set_grain_len_max(&mut self, grain_len_max_in_ms: u32) -> &mut Self {
        self.grain_len_max_raw = grain_len_max_in_ms;
        self.grain_len_max = self.sanitize_grain_len_max(grain_len_max_in_ms as u32);

        // decrease current grain length min to be less than the new max
        if self.grain_len_max < self.grain_len_min {
            self.set_grain_len_min(
                self.grain_len_max - GranularSynthesizer::GRAIN_LEN_ABSOLUTE_MIN_IN_MS,
            );
        }

        // prevent long grains from lingering when grain length has changed
        self.filter_long_grains();

        self
    }

    fn set_max_number_of_channels(&mut self, max_num_channels: u32) -> &mut Self {
        self.max_num_channels = max_num_channels;

        // assumption: it's ok for the `grains`, `output_buffer_samples`, and `output_env_samples`
        // vectors to be LONGER than `max_num_channels`, because they are not used as the basis of iteration
        // i.e. we only ever iterate using `max_num_channels`, so `max_num_channels` can be <= the other vectors

        let max_num_channels = max_num_channels as usize;

        // extend grains to be as long as max number of channels
        if max_num_channels > self.grains.len() {
            let num_extra_samples = max_num_channels - self.grains.len();
            self.grains
                .extend(vec![GranularSynthesizer::new_grain(); num_extra_samples]);
        }

        // extend samples buffer to be as long as max number of channels
        if max_num_channels > self.output_buffer_samples.len() {
            let num_extra_samples = max_num_channels - self.output_buffer_samples.len();
            self.output_buffer_samples
                .extend(vec![0.0; num_extra_samples]);
        }

        // extend envelope buffer to be as long as max number of channels
        if max_num_channels > self.output_env_samples.len() {
            let num_extra_samples = max_num_channels - self.output_env_samples.len();
            self.output_env_samples.extend(vec![0.0; num_extra_samples]);
        }

        self
    }

    fn set_density(&mut self, density: f32) -> &mut Self {
        self.density = Self::sanitize_density(density);

        self
    }

    fn set_buffer(&mut self, buffer: Arc<Vec<f32>>) -> &mut Self {
        let buffer_len_samples = buffer.len();
        self.buffer = buffer;

        // replace any buffers that extend past the current buffer length
        for grain in &mut self.grains {
            if grain.end_frame > buffer_len_samples
                || grain.start_frame > buffer_len_samples
                || grain.len > buffer_len_samples
                || grain.current_frame > buffer_len_samples
            {
                grain.finished = true;
            }
        }

        // reinitialize grain lengths constraints since buffer length could now be different
        self.set_grain_len_max(self.grain_len_max_raw);

        self.set_grain_len_min(self.grain_len_min_raw);

        self
    }

    fn next_frame(&mut self) -> Vec<f32> {
        self.refresh_grains();
        self.fill_buffer_and_env_samples();
        self.get_frame_data()
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = sample_rate;

        self
    }

    fn get_grain_len_min(&self) -> u32 {
        self.grain_len_min
    }

    fn get_grain_len_max(&self) -> u32 {
        self.grain_len_max
    }
}

// internal logic to support public GranularSynthesizer interface
impl GranularSynthesizer {
    fn sanitize_grain_len_min(&self, grain_len_min: u32) -> u32 {
        grain_len_min
            // should be smaller than existing max
            .min(self.grain_len_max - Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS)
            // shold be >= smallest possible length
            .max(Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS)
    }

    fn sanitize_grain_len_max(&self, grain_len_max: u32) -> u32 {
        grain_len_max
            // should be bigger than existing min
            .max(self.grain_len_min + Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS)
            // new max len should not be longer than the length of the buffer
            .min(self.get_buffer_len_in_ms())
            // new max should be greater than the smallest possible min length
            .max(Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS + Self::GRAIN_LEN_ABSOLUTE_MIN_IN_MS)
    }

    fn get_buffer_len_in_ms(&self) -> u32 {
        let buffer_len = self.buffer.len();

        if buffer_len == 0 {
            return 0;
        }

        (buffer_len as u32 / self.sample_rate) * 1000
    }

    fn get_grain_len_min_samples(&self) -> u32 {
        if self.grain_len_min == 0 {
            return 0;
        }

        let grain_len_min_second_percent = self.grain_len_min as f64 / 1000.0;
        (grain_len_min_second_percent * self.sample_rate as f64) as u32
    }

    fn get_grain_len_max_samples(&self) -> u32 {
        if self.grain_len_max == 0 {
            return 0;
        }

        let grain_len_max_second_percent = self.grain_len_max as f64 / 1000.0;
        (grain_len_max_second_percent * self.sample_rate as f64) as u32
    }

    /// Iterates through array of grains (1 grain for each channel), and refreshes any
    /// grains that were previously finished with a new range of buffer indexes.
    fn refresh_grains(&mut self) {
        // get start and end of selection
        let selection_start_index = self.get_selection_start_in_samples();
        let selection_end_index = self.get_selection_end_in_samples();

        let grain_len_max_samples = self.get_grain_len_max_samples();
        let grain_len_min_samples = self.get_grain_len_min_samples();

        let selection_len = selection_end_index - selection_start_index;

        // get largest possible grain length:
        // the smaller value between (selection_end - selection_start) & grain_len_max]
        let largest_possible_grain_len = selection_len.min(grain_len_max_samples);

        // get smallest possible grain length:
        // the smaller value between (selection_end - selection_start) & grain_len_min
        let smallest_possible_grain_len = selection_len.min(grain_len_min_samples);

        let largest_starting_index =
            (selection_end_index - smallest_possible_grain_len).max(selection_start_index);

        // if nothing is selected, there's no use in refreshing grains with empty data
        let selection_is_empty = selection_start_index >= selection_end_index
            || selection_start_index >= largest_starting_index
            || smallest_possible_grain_len >= largest_possible_grain_len;

        if !selection_is_empty {
            for grain in self.grains.iter_mut() {
                if grain.finished {
                    // get random index inside selection
                    let start_index = self
                        .rng
                        .gen_range(selection_start_index..=largest_starting_index);

                    // get random length
                    let random_length = self
                        .rng
                        .gen_range(smallest_possible_grain_len..=largest_possible_grain_len);

                    // constrain end index based on selection end
                    let end_index = (start_index + random_length).min(selection_end_index);

                    let new_grain = Grain::new(start_index as usize, end_index as usize);
                    *grain = new_grain;
                }
            }
        }
    }

    fn get_selection_start_in_samples(&self) -> u32 {
        (self.selection_start * self.buffer.len() as f32) as u32
    }

    fn get_selection_end_in_samples(&self) -> u32 {
        (self.selection_end * self.buffer.len() as f32) as u32
    }

    /// Prevent long grains from lingering when max length and/or selection has changed
    fn filter_long_grains(&mut self) {
        let grain_len_max_in_samples = self.get_grain_len_max_samples() as usize;
        let selection_len_in_samples =
            (self.get_selection_end_in_samples() - self.get_selection_start_in_samples()) as usize;

        for grain in self.grains.iter_mut() {
            let remaining_grain_samples = grain.end_frame - grain.current_frame;
            if remaining_grain_samples > grain_len_max_in_samples
                || remaining_grain_samples > selection_len_in_samples
            {
                grain.finished = true;
            }
        }
    }

    /// Fills in buffer & envelope sample data for each channel
    /// based on the current state of the grain for each channel.
    ///
    /// Each buffer sample and envelope sample must be coordinated/aligned to prevent
    /// audio clipping and/or unexpected audio results.
    fn fill_buffer_and_env_samples(&mut self) {
        // get value of each grain's current index in the buffer for each channel
        self.grains.iter_mut().enumerate().for_each(|(i, grain)| {
            // this can happen if a grain has finished, but the selection
            // size is 0, so the grain can't get refreshed with more data
            if grain.finished {
                self.output_buffer_samples[i] = 0.0;
                self.output_env_samples[i] = 0.0;
                return;
            }

            let envelope_percent =
                ((grain.current_frame - grain.start_frame) as f32) / (grain.len as f32);

            let envelope_value =
                utils::generate_triangle_envelope_value_from_percent(envelope_percent);
            let frame_index = grain.current_frame;
            let sample_value = self.buffer[frame_index];

            self.output_buffer_samples[i] = sample_value;
            self.output_env_samples[i] = envelope_value;

            grain.get_next_frame();
        });
    }

    /// this represents the number of channels actually in use
    fn get_num_channels_for_frame(&self) -> usize {
        (self.max_num_channels as f32 * self.density) as usize
    }

    /// Combines current buffer and envelope sample values to calculate a full audio frame
    /// (where each channel gets a single audio output value).
    fn get_frame_data(&self) -> Vec<f32> {
        let num_channels_for_frame = self.get_num_channels_for_frame();
        let mut frame_data = vec![0.0; num_channels_for_frame];
        for (i, channel) in frame_data.iter_mut().enumerate() {
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
        frame_data
    }
}
