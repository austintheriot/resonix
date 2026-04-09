use alloc::vec::Vec;

use crate::{errors::AudioBufferError, primitives::Sample, traits::AudioBuffer};

fn samples_vec_from(values: &[f32]) -> Vec<Sample> {
    values.iter().map(|&v| Sample::from(v)).collect()
}

// --- AudioBuffer::block_size ---

pub fn test_block_size_mono<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0, 2.0, 3.0, 4.0]);
    let buf = make(&data, 1);
    assert_eq!(buf.block_size(), 4);
}

pub fn test_block_size_stereo<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0, 2.0, 3.0, 4.0]);
    let buf = make(&data, 2);
    assert_eq!(buf.block_size(), 2);
}

pub fn test_block_size_empty<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data: Vec<Sample> = Vec::new();
    let buf = make(&data, 1);
    assert_eq!(buf.block_size(), 0);
}

// --- AudioBuffer::channels ---

pub fn test_channels_mono<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0]);
    let buf = make(&data, 1);
    assert_eq!(buf.channels(), 1);
}

pub fn test_channels_four<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0, 2.0, 3.0, 4.0]);
    let buf = make(&data, 4);
    assert_eq!(buf.channels(), 4);
}

// --- AudioBuffer::channel ---

pub fn test_channel_returns_correct_slice<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    // Stereo, 2 samples per channel: [L0, L1, R0, R1]
    let data = samples_vec_from(&[1.0, 2.0, 3.0, 4.0]);
    let buf = make(&data, 2);
    let ch0 = samples_vec_from(&[1.0, 2.0]);
    let ch1 = samples_vec_from(&[3.0, 4.0]);
    assert_eq!(buf.channel(0usize).unwrap(), ch0.as_slice());
    assert_eq!(buf.channel(1usize).unwrap(), ch1.as_slice());
}

pub fn test_channel_out_of_range<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0, 2.0]);
    let buf = make(&data, 1);
    assert_eq!(
        buf.channel(1usize).unwrap_err(),
        AudioBufferError::ChannelOutOfRange {
            index: 1,
            channels: 1,
        }
    );
}

// --- AudioBuffer::channels_iter ---

pub fn test_channels_iter_single_channel<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[0.0, 1.0, 2.0, 3.0]);
    let buf = make(&data, 1);
    let mut iter = buf.channels_iter().unwrap();
    assert_eq!(iter.next().unwrap(), data.as_slice());
    assert!(iter.next().is_none());
}

pub fn test_channels_iter_four_channels<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[0.0, 1.0, 2.0, 3.0]);
    let buf = make(&data, 4);
    let mut iter = buf.channels_iter().unwrap();
    assert_eq!(iter.next().unwrap(), &data[0..1]);
    assert_eq!(iter.next().unwrap(), &data[1..2]);
    assert_eq!(iter.next().unwrap(), &data[2..3]);
    assert_eq!(iter.next().unwrap(), &data[3..4]);
    assert!(iter.next().is_none());
}

// --- AudioBuffer::mono ---

pub fn test_mono_returns_samples<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0, 2.0, 3.0]);
    let buf = make(&data, 1);
    assert_eq!(buf.mono().unwrap(), data.as_slice());
}

pub fn test_mono_stereo_returns_error<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0, 2.0]);
    let buf = make(&data, 2);
    assert_eq!(
        buf.mono().unwrap_err(),
        AudioBufferError::NotMono { channels: 2 }
    );
}

// --- AudioBuffer::as_slice ---

pub fn test_as_slice_returns_all_samples<A: AudioBuffer>(make: impl Fn(&[Sample], usize) -> A) {
    let data = samples_vec_from(&[1.0, 2.0, 3.0, 4.0]);
    let buf = make(&data, 2);
    assert_eq!(buf.as_slice(), data.as_slice());
}
