use hound::{WavSpec, WavWriter};
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

/// Holds raw `f32` sample data and exposes utilities for converting
/// that sample data to .wav file and downloading it
#[derive(Default, Debug, Clone)]
pub struct AudioRecorderHandle {
    data: Arc<Mutex<Vec<f32>>>,
}

impl PartialEq for AudioRecorderHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl Extend<f32> for AudioRecorderHandle {
    /// Extends inner data buffer with the contents of an iterator containing sample data
    fn extend<T: IntoIterator<Item = f32>>(&mut self, iter: T) {
        self.data.lock().unwrap().extend(iter)
    }
}

impl AudioRecorderHandle {
    /// Returns the audio samples, encoded as a .wav binary file
    pub fn encode_as_wav(&self, spec: WavSpec) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut bytes_cursor = Cursor::new(&mut bytes);
        let mut wav_writer = WavWriter::new(&mut bytes_cursor, spec).unwrap();
        let stored_sample_data = self.data.lock().unwrap();
        let amplitude = i16::MAX as f32;
        for sample in stored_sample_data.iter() {
            wav_writer
                .write_sample((sample * amplitude) as i16)
                .unwrap();
        }
        wav_writer.finalize().unwrap();
        bytes
    }
}
