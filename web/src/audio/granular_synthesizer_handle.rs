use common::granular_synthesizer::GranularSynthesizer;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub const MAX_NUM_CHANNELS: usize = 250;
pub const GRAIN_LEN_MIN_IN_MS: usize = 10;
pub const GRAIN_LEN_MAX_IN_MS: usize = 1000;

#[derive(Clone)]
pub struct GranularSynthesizerHandle {
    granular_synthesizer: Arc<Mutex<GranularSynthesizer>>,
    counter: u8,
    uuid: Uuid,
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

pub fn init_granular_synth(
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

impl GranularSynthesizerHandle {
    pub fn new(buffer: Arc<Vec<f32>>, sample_rate: u32) -> GranularSynthesizerHandle {
        Self {
            granular_synthesizer: Arc::new(Mutex::new(GranularSynthesizer::new(
                buffer,
                sample_rate,
            ))),
            counter: Default::default(),
            uuid: Uuid::new_v4(),
        }
    }

    /// Replaces the current granular synthesizer with a new one, based on a new buffer and/or sample rate.
    ///
    /// This is useful, because it allows the audio thread to know which granular synthesizer to pull frames,
    /// and also allows the UI to update the buffer / granular synthesizer that is being read from.
    pub fn replace(&self, buffer: Arc<Vec<f32>>, sample_rate: u32) {
        *self.granular_synthesizer.lock().unwrap() = init_granular_synth(buffer, sample_rate);
    }

    pub fn set_selection_start(&self, start: f32) -> &Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_selection_start(start);

        self
    }

    pub fn set_selection_end(&self, end: f32) -> &Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_selection_end(end);

        self
    }

    pub fn set_density(&self, density: f32) -> &Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_density(density);

        self
    }

    pub fn next_frame(&self) -> Vec<f32> {
        self.granular_synthesizer.lock().unwrap().next_frame()
    }
}
