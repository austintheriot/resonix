use super::defaults::{GRAIN_LEN_MAX_IN_MS, GRAIN_LEN_MIN_IN_MS, MAX_NUM_CHANNELS};
use common::granular_synthesizer::GranularSynthesizer;
use common::granular_synthesizer_action::GranularSynthesizerAction;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Wrapper around GranularSynthesizer for accessing data from both the UI and the audio thread.
#[derive(Clone)]
pub struct GranularSynthesizerHandle {
    granular_synthesizer: Arc<Mutex<GranularSynthesizer>>,
    counter: u8,
    uuid: Uuid,
}

impl GranularSynthesizerHandle {
    /// Replaces the current granular synthesizer with a new one, based on a new buffer and/or sample rate.
    ///
    /// This is useful, because it allows the audio thread to know which granular synthesizer to pull frames from,
    /// and also allows the UI to update the buffer / granular synthesizer that is being read from.
    pub fn replace(&self, buffer: Arc<Vec<f32>>, sample_rate: u32) {
        *self.granular_synthesizer.lock().unwrap() =
            Self::new_granular_synthesizer_with_defaults(buffer, sample_rate);
    }

    /// Creates a new GranularSynthesizer instance and updates it with any necessary defaults
    pub fn new_granular_synthesizer_with_defaults(
        mp3_source_data: Arc<Vec<f32>>,
        sample_rate: u32,
    ) -> GranularSynthesizer {
        let mut granular_synth: GranularSynthesizer =
            GranularSynthesizer::new(mp3_source_data, sample_rate);

        // this data does not need to be updated dynamically (for now at least)
        granular_synth
            .set_grain_len_min(GRAIN_LEN_MIN_IN_MS)
            .set_grain_len_max(GRAIN_LEN_MAX_IN_MS)
            .set_max_number_of_channels(MAX_NUM_CHANNELS);

        granular_synth
    }
}

impl GranularSynthesizerAction for GranularSynthesizerHandle {
    const DENSITY_MAX: f32 = 1.0;

    const DENSITY_MIN: f32 = 0.0;

    const DEFAULT_NUM_CHANNELS: usize = 2;

    const DEFAULT_DENSITY: f32 = 1.0;

    const GRAIN_MIN_LEN_IN_MS: u32 = 1;

    fn new(buffer: Arc<Vec<f32>>, sample_rate: u32) -> GranularSynthesizerHandle {
        Self {
            granular_synthesizer: Arc::new(Mutex::new(GranularSynthesizer::new(
                buffer,
                sample_rate,
            ))),
            counter: Default::default(),
            uuid: Uuid::new_v4(),
        }
    }

    fn set_selection_start(&mut self, start: f32) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_selection_start(start);

        self
    }

    fn set_selection_end(&mut self, end: f32) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_selection_end(end);

        self
    }

    fn set_density(&mut self, density: f32) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_density(density);

        self
    }

    fn next_frame(&mut self) -> Vec<f32> {
        self.granular_synthesizer.lock().unwrap().next_frame()
    }

    fn get_grain_len_min_decimal(&self) -> f32 {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .get_grain_len_min_decimal()
    }

    fn get_grain_len_smallest_samples(&self) -> u32 {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .get_grain_len_smallest_samples()
    }

    fn set_grain_len_min(&mut self, input_min_len_in_ms: usize) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_grain_len_min(input_min_len_in_ms);

        self
    }

    fn set_grain_len_max(&mut self, input_max_len_in_ms: usize) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_grain_len_max(input_max_len_in_ms);

        self
    }

    fn set_max_number_of_channels(&mut self, max_num_channels: usize) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_max_number_of_channels(max_num_channels);

        self
    }
}

impl PartialEq for GranularSynthesizerHandle {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter && self.uuid == other.uuid
    }
}

impl Debug for GranularSynthesizerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GranularSynthesizerHandle")
            .field("counter", &self.counter)
            .field("uuid", &self.uuid)
            .finish()
    }
}
