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
    max_num_channels: usize,

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

    /// length in samples
    grain_len_min: u32,

    /// length in samples
    grain_len_max: u32,

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
    const DENSITY_MAX: f32 = 1.0;

    const DENSITY_MIN: f32 = 0.0;

    const DEFAULT_NUM_CHANNELS: usize = 2;

    const DEFAULT_DENSITY: f32 = 1.0;

    const GRAIN_MIN_LEN_IN_MS: u32 = 1;

    fn new(buffer: Arc<Vec<f32>>, sample_rate: u32) -> Self {
        let buffer_len = buffer.len();
        GranularSynthesizer {
            sample_rate,
            buffer: buffer,
            grains: vec![
                GranularSynthesizer::new_grain();
                GranularSynthesizer::DEFAULT_NUM_CHANNELS
            ],
            rng: rand::rngs::StdRng::from_entropy(),
            grain_len_min: sample_rate / (1000 / GranularSynthesizer::GRAIN_MIN_LEN_IN_MS),
            grain_len_max: buffer_len as u32,
            output_buffer_samples: vec![0.0; GranularSynthesizer::DEFAULT_NUM_CHANNELS],
            output_env_samples: vec![0.0; GranularSynthesizer::DEFAULT_NUM_CHANNELS],
            selection_start: 0.0,
            selection_end: 0.0,
            max_num_channels: GranularSynthesizer::DEFAULT_NUM_CHANNELS,
            density: GranularSynthesizer::DEFAULT_DENSITY,
        }
    }

    fn get_grain_len_min_decimal(&self) -> f32 {
        self.grain_len_min as f32 / self.buffer.len() as f32
    }

    fn get_grain_len_smallest_samples(&self) -> u32 {
        self.sample_rate / (1000 / GranularSynthesizer::GRAIN_MIN_LEN_IN_MS)
    }

    fn set_selection_start(&mut self, start: f32) -> &mut Self {
        let grain_len_min_decimal = self.get_grain_len_min_decimal();
        let sanitized_start = start.max(0.0).min(1.0).min(1.0 - grain_len_min_decimal);
        self.selection_start = sanitized_start;
        if sanitized_start > self.selection_end {
            // move end to "catch up" to the beginning
            self.set_selection_end(sanitized_start + grain_len_min_decimal);
        }
        self
    }

    fn set_selection_end(&mut self, start: f32) -> &mut Self {
        let grain_len_min_decimal = self.get_grain_len_min_decimal();
        let sanitized_end = start.max(0.0).min(1.0).max(0.0 + grain_len_min_decimal);
        self.selection_end = sanitized_end;
        if sanitized_end < self.selection_start {
            // move beginning to be before the ending
            self.set_selection_start(sanitized_end - grain_len_min_decimal);
        }
        self
    }

    fn set_grain_len_min(&mut self, input_min_len_in_ms: usize) -> &mut Self {
        // the smallest accetable length
        let min_len_in_samples = self.get_grain_len_smallest_samples();

        let input_min_len_in_samples = self.sample_rate / (1000 / input_min_len_in_ms as u32);
        self.grain_len_min = input_min_len_in_samples
            // min should be less than existing max
            .min(self.grain_len_max - GranularSynthesizer::GRAIN_MIN_LEN_IN_MS)
            // min len should not be less than the end of the smallest possible grain
            .max(min_len_in_samples);

        self
    }

    fn set_grain_len_max(&mut self, input_max_len_in_ms: usize) -> &mut Self {
        let input_max_len_in_samples = self.sample_rate / (1000 / input_max_len_in_ms as u32);
        self.grain_len_max = input_max_len_in_samples
            // max should be greater than existing min
            .max(self.grain_len_min + GranularSynthesizer::GRAIN_MIN_LEN_IN_MS)
            // max len should not be longer than the length of the buffer
            .min(self.buffer.len() as u32);

        self
    }

    fn set_max_number_of_channels(&mut self, max_num_channels: usize) -> &mut Self {
        self.max_num_channels = max_num_channels;

        // assumption: it's ok for the `grains`, `output_buffer_samples`, and `output_env_samples`
        // vectors to be LONGER than `max_num_channels`, because they are not used as the basis of iteration

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
        let buffer_len = self.buffer.len();
        
        // replace any buffers that extend past the current buffer length
        for grain in &mut self.grains {
            if grain.end_frame > buffer_len
            || grain.start_frame > buffer_len
            || grain.len > buffer_len
            || grain.current_frame > buffer_len
            {
                grain.finished = true;
            }
        }
        
        self.grain_len_max = buffer_len as u32;
        self.buffer = buffer;

        self
    }

    fn next_frame(&mut self) -> Vec<f32> {
        self.refresh_grains();
        self.fill_buffer_and_env_samples();
        self.get_frame_data()
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = sample_rate;

        // @todo: update min/max sample length here once
        // original min/max length inputs are saved?
        // (to prevent having to reinitialize every time)

        self
    }
}

// internal logic to support public GranularSynthesizer interface
impl GranularSynthesizer {
    /// Iterates through array of grains (1 grain for each channel), and refreshes any
    /// grains that were previously finished with a new range of buffer indexes.
    fn refresh_grains(&mut self) {
        for grain in self.grains.iter_mut() {
            if grain.finished {
                let selection_start_in_samples = ((self.selection_start * self.buffer.len() as f32)
                    as usize)
                    .min(self.buffer.len() - self.grain_len_min as usize);
                let selection_end_in_samples = ((self.selection_end * self.buffer.len() as f32)
                    as usize)
                    .max(selection_start_in_samples + self.grain_len_min as usize);

                let smallest_range = if (self.grain_len_max as usize - self.grain_len_min as usize)
                    < selection_end_in_samples - selection_start_in_samples
                {
                    self.grain_len_min as usize..self.grain_len_max as usize
                } else {
                    selection_start_in_samples..selection_end_in_samples
                };

                let envolope_len_samples = self
                    .rng
                    .gen_range(smallest_range)
                    .min(selection_end_in_samples - selection_start_in_samples);

                let max_index_by_len = self.buffer.len() - envolope_len_samples;
                let max_index = max_index_by_len.min(selection_end_in_samples);

                let start_frame = self.rng.gen_range(selection_start_in_samples..max_index);
                let end_frame = start_frame + envolope_len_samples;

                let new_grain = Grain::new(start_frame as usize, end_frame as usize);
                *grain = new_grain;
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
        let num_channels = (self.max_num_channels as f32 * self.density) as usize;
        num_channels
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
                .map(|value| *value)
                .expect("output_buffer_samples length should match max number of grains");

            let envelope_sample = self
                .output_env_samples
                .get(i)
                .map(|value| *value)
                .expect("output_env_samples length should match max number of grains");

            // if these buffers have not been filled up yet, just return 0.0
            *channel = buffer_sample * envelope_sample;
        }
        frame_data
    }
}
