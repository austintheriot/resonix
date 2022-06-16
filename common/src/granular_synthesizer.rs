use crate::grain::Grain;
use crate::utils;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;

/// Accepts a reference to a buffer of Vec<f32> audio sample data.
///
/// Generates random multi-channel audio grain output.
pub struct GranularSynthesizer<const NUM_CHANNELS: usize = 2> {
    sample_rate: u32,
    buffer: Arc<Vec<f32>>,
    /// List of grains and their current progress through the buffer.
    ///
    /// 1 array element = 1 grain = 1 channel of audio
    grains: [Grain; NUM_CHANNELS],
    /// used to generate random indexes
    rng: StdRng,
    /// length in samples
    grain_len_min: u32,
    /// length in samples
    grain_len_max: u32,
    /// Samples that have been copied over from the audio buffer.
    ///
    /// each array value = 1 channel
    output_buffer_samples: [f32; NUM_CHANNELS],
    /// Envelope values that have been copied over to match the
    /// current progress of a grain.
    ///
    /// each array value = 1 channel
    output_env_samples: [f32; NUM_CHANNELS],
}

/// Produces an unitialized grain for filling the initial array of Grains
///
/// Once it is time to actually produce an audio sample from the buffer,
/// each grain will be initialized with a randaom start/end index, etc.
const fn new_grain() -> Grain {
    Grain {
        current_frame: 0,
        end_frame: 0,
        finished: true,
        len: 0,
        start_frame: 0,
    }
}

impl<const C: usize> GranularSynthesizer<C> {
    /// Creates a new Granular Synthesizer instance
    pub fn new(buffer: Arc<Vec<f32>>, sample_rate: u32) -> Self {
        let buffer_len = buffer.len();
        GranularSynthesizer {
            sample_rate,
            buffer: buffer,
            grains: [new_grain(); C],
            rng: rand::rngs::StdRng::from_entropy(),
            grain_len_min: 1,
            grain_len_max: buffer_len as u32,
            output_buffer_samples: [0.0; C],
            output_env_samples: [0.0; C],
        }
    }

    /// The smallest possible grain length is 1 ms,
    /// and the min grain length and can never exceed the max.
    pub fn set_grain_len_min(mut self, input_len_in_ms: usize) -> Self {
        // the smallest accetable length
        let min_len_in_ms = 1;
        let min_len_in_samples = self.sample_rate / (1000 / min_len_in_ms);

        let input_len_in_samples = self.sample_rate / (1000 / input_len_in_ms as u32);
        self.grain_len_min = input_len_in_samples
            // min should be less than existing max
            .min(self.grain_len_max - 1)
            // min should not be less than the equivalent of 1ms in samples
            .max(min_len_in_samples);

        self
    }

    /// The maximum grain length is the size of the buffer itself,
    /// and max grain length can never be lower than the min width (or 1ms--whichever is higher)
    pub fn set_grain_len_max(mut self, input_len_in_ms: usize) -> Self {
        let input_len_in_samples = self.sample_rate / (1000 / input_len_in_ms as u32);
        self.grain_len_max = input_len_in_samples
            // max should be greater than existing min
            .max(self.grain_len_min + 1)
            // max should not be longer than the length of the buffer
            .min(self.buffer.len() as u32);
        self
    }

    /// Iterates through array of grains (1 grain for each channel), and refreshes any
    /// grains that were previously finished with a new range of buffer indexes.
    fn refresh_grains(&mut self) {
        for grain in self.grains.iter_mut() {
            if grain.finished {
                let envolope_len_samples = self
                    .rng
                    .gen_range(self.grain_len_min as usize..self.grain_len_max as usize);
                let max_index = self.buffer.len() - envolope_len_samples;
                let start_frame = self.rng.gen_range(0..max_index as usize);
                let end_frame = start_frame + envolope_len_samples;

                debug_assert!(start_frame > 0);
                debug_assert!(end_frame > 0);
                debug_assert!(start_frame < self.buffer.len());
                debug_assert!(end_frame < self.buffer.len());

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
            debug_assert_eq!(grain.finished, false);

            let envelope_percent =
                ((grain.current_frame - grain.start_frame) as f32) / (grain.len as f32);
            debug_assert!(envelope_percent >= 0.0, "{}", envelope_percent);
            debug_assert!(envelope_percent < 1.0, "{}", envelope_percent);

            let envelope_value =
                utils::generate_triangle_envelope_value_from_percent(envelope_percent);
            let frame_index = grain.current_frame;
            let sample_value = self.buffer[frame_index];

            self.output_buffer_samples[i] = sample_value;
            self.output_env_samples[i] = envelope_value;

            grain.get_next_frame();
        });
    }

    /// Combines current buffer and envelope sample values to calculate a full audio frame
    /// (where each channel gets a single audio output value).
    fn get_frame_data(&self) -> [f32; C] {
        let mut frame_data = [0.0; C];
        for (i, channel) in frame_data.iter_mut().enumerate() {
            *channel = self.output_buffer_samples[i] * self.output_env_samples[i];
        }
        frame_data
    }

    /// Returns a full audio frame (1 array element = 1 audio channel value),
    /// where each channel gets its own, indepedent value
    /// based on the progression of that audio channel's grain.
    pub fn next_frame(&mut self) -> [f32; C] {
        self.refresh_grains();
        self.fill_buffer_and_env_samples();
        self.get_frame_data()
    }
}
