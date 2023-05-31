use audio::granular_synthesizer::GranularSynthesizer;
use audio::granular_synthesizer::GranularSynthesizerAction;
use audio::Percentage;
use audio::{EnvelopeType, NumChannels};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Wrapper around GranularSynthesizer for accessing data from both the UI and the audio thread.
#[derive(Clone)]
pub struct GranularSynthesizerHandle {
    granular_synthesizer: Arc<Mutex<GranularSynthesizer>>,
    counter: u8,
    uuid: Uuid,
}

impl GranularSynthesizerAction for GranularSynthesizerHandle {
    fn new() -> GranularSynthesizerHandle {
        Self {
            granular_synthesizer: Arc::new(Mutex::new(GranularSynthesizer::new())),
            counter: Default::default(),
            uuid: Uuid::new_v4(),
        }
    }

    fn selection_start(&self) -> Percentage {
        self.granular_synthesizer.lock().unwrap().selection_start()
    }

    fn set_selection_start(&mut self, start: impl Into<Percentage>) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_selection_start(start);

        self
    }

    fn selection_end(&self) -> Percentage {
        self.granular_synthesizer.lock().unwrap().selection_end()
    }

    fn set_selection_end(&mut self, end: impl Into<Percentage>) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_selection_end(end);

        self
    }

    fn set_grain_len(&mut self, grain_len: impl Into<Duration>) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_grain_len(grain_len);

        self
    }

    fn set_num_channels(&mut self, channels: impl Into<NumChannels>) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_num_channels(channels);

        self
    }

    fn num_channels(&self) -> NumChannels {
        self.granular_synthesizer.lock().unwrap().num_channels()
    }

    fn set_buffer(&mut self, buffer: Arc<Vec<f32>>) -> &mut Self {
        self.granular_synthesizer.lock().unwrap().set_buffer(buffer);

        self
    }

    fn next_frame_into_buffer<'a>(
        &mut self,
        frame_data_buffer: &'a mut Vec<f32>,
    ) -> &'a mut Vec<f32> {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .next_frame_into_buffer(frame_data_buffer)
    }

    fn next_frame(&mut self) -> Vec<f32> {
        self.granular_synthesizer.lock().unwrap().next_frame()
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_sample_rate(sample_rate);

        self
    }

    fn grain_len(&self) -> Duration {
        self.granular_synthesizer.lock().unwrap().grain_len()
    }

    fn set_envelope(&mut self, envelope_type: EnvelopeType) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_envelope(envelope_type);

        self
    }

    fn set_grain_initialization_delay(&mut self, delay: impl Into<Duration>) -> &mut Self {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .set_grain_initialization_delay(delay);
        self
    }

    fn grain_initialization_delay(&self) -> Duration {
        self.granular_synthesizer
            .lock()
            .unwrap()
            .grain_initialization_delay()
    }
}

impl Default for GranularSynthesizerHandle {
    /// Instantiate with global app audio defaults
    fn default() -> GranularSynthesizerHandle {
        let granular_synth: GranularSynthesizer = GranularSynthesizer::new();

        Self {
            granular_synthesizer: Arc::new(Mutex::new(granular_synth)),
            counter: Default::default(),
            uuid: Uuid::new_v4(),
        }
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
