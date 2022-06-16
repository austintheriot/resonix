use std::sync::Arc;

use crate::grain::Grain;
use crate::utils;
use rand::prelude::{StdRng};
use rand::{Rng, SeedableRng};

/// Accepts a reference to any type of buffer than can be indexed
///
pub struct GranularSynthesizer<const C: usize = 2>
{   
    sample_rate: u32,
    buffer: Arc<Vec<f32>>,
    grains: [Grain; C],
    /// used to generate random indexes
    rng: StdRng,
    /// length in samples
    grain_len_min: u32,
    /// length in samples
    grain_len_max: u32,
    buffer_samples: [f32; C],
    envelope_samples: [f32; C]
}

/// Produces an unitialized grain for filling the initial Grain array
const fn new_grain() -> Grain {
    Grain {
        current_frame: 0,
        end_frame: 0,
        finished: true,
        len: 0,
        start_frame: 0,
    }
}

impl<const C: usize> GranularSynthesizer<C>
     {
    pub fn new(buffer: Arc<Vec<f32>>, sample_rate: u32) -> Self {
        let buffer_len =  buffer.len();
        GranularSynthesizer {
            sample_rate,
            buffer: buffer,
            grains: [new_grain(); C],
            rng: rand::rngs::StdRng::from_entropy(),
            grain_len_min: 1,
            grain_len_max: buffer_len as u32,
            buffer_samples: [0.0; C],
            envelope_samples: [0.0; C],
        }
    }

    pub fn set_grain_len_min(mut self, input_len_in_ms: usize) -> Self {
        let min_len_in_ms = 1;
        let min_len_in_samples = self.sample_rate / (1000 / min_len_in_ms);

        let input_len_in_samples = self.sample_rate / (1000 / input_len_in_ms as u32);
        self.grain_len_min = input_len_in_samples.max(min_len_in_samples);
        self
    }

    pub fn set_grain_len_max(mut self, input_len_in_ms: usize) -> Self {
        let input_len_in_samples = self.sample_rate / (1000 / input_len_in_ms as u32);
        self.grain_len_max = input_len_in_samples.min(self.buffer.len() as u32);
        self
    }
    

    /// Iterates through array of grains (1 for each channel), and refreshes any
    /// grains that are now finished.
    fn refresh_grains(&mut self) {
        for grain in self.grains.iter_mut() {
            if grain.finished {
                let envolope_len_samples =
                self.rng.gen_range(self.grain_len_min as usize..self.grain_len_max as usize);
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
    fn fill_buffer_and_env_samples(&mut self) {
        // get value of each grain's current index in the buffer for each channel
        self.grains
            .iter_mut()
            .enumerate()
            .for_each(|(i, grain)| {
                debug_assert_eq!(grain.finished, false);

                let envelope_percent =
                    ((grain.current_frame - grain.start_frame) as f32) / (grain.len as f32);
                debug_assert!(envelope_percent >= 0.0, "{}", envelope_percent);
                debug_assert!(envelope_percent < 1.0, "{}", envelope_percent);

                let envelope_value =
                    utils::generate_triangle_envelope_value_from_percent(envelope_percent);
                let frame_index = grain.current_frame;
                let sample_value = self.buffer[frame_index];

                self.buffer_samples[i] = sample_value;
                self.envelope_samples[i] = envelope_value;

                grain.get_next_frame();
            });
    }

    /// Uses current buffer and envelope sample values to calculate a frame
    fn get_frame_data(&self) -> [f32; C] {
        let mut frame_data = [0.0; C];
        for (i, channel) in frame_data.iter_mut().enumerate() {
            *channel = self.buffer_samples[i] * self.envelope_samples[i];
        } 
        frame_data
    }

    /// returns the next frame for each channel,
    /// where each channel's value represents a value in the array
    pub fn next_frame(&mut self) -> [f32; C] {
        self.refresh_grains();
        self.fill_buffer_and_env_samples();
        self.get_frame_data()
    }
}
