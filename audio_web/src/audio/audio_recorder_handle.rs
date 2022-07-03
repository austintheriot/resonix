use hound::{SampleFormat, WavSpec, WavWriter};
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

use crate::utils::download;

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
    /// Downloads the audio samples, encoded as a .wav binary file
    pub fn download_as_wav(&self, num_channels: impl Into<u16>, sample_rate: impl Into<u32>) {
        let wav_bytes = self.encode_as_wav(num_channels, sample_rate);
        download::download_bytes(wav_bytes, "recording.wav");
    }

    /// Returns the audio samples, encoded as a .wav binary file
    pub fn encode_as_wav(
        &self,
        num_channels: impl Into<u16>,
        sample_rate: impl Into<u32>,
    ) -> Vec<u8> {
        let wav_spec = WavSpec {
            channels: num_channels.into(),
            sample_rate: sample_rate.into(),
            // these were the default used in the `hound` doc examples
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut bytes = Vec::new();
        let mut bytes_cursor = Cursor::new(&mut bytes);
        let mut wav_writer = WavWriter::new(&mut bytes_cursor, wav_spec).unwrap();
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
