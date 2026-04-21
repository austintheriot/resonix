use core::ptr::NonNull;

use crate::errors::AudioBufferError;
use crate::primitives::{Channel, Sample};
use crate::traits::{AudioBuffer, AudioBufferMut};

/// Non-owning, non-lifetimed, unsafe pointer to a multi-channel audio buffer.
///
/// Gives flexibility to accessing audio buffers when ownership cannot be
/// statically guaranteed (such as during audio graph execution).
///
/// `Option<RawAudioBuffer>` uses the null-pointer niche of `ptr` (the first
/// field), giving it the same size as three `usize`s with no discriminant.
#[repr(C)]
#[derive(Debug)]
pub struct RawAudioBuffer {
    /// Fat pointer: data pointer + (block_size * channels) as the length.
    pub ptr: NonNull<[Sample]>,
    pub channels: usize,
}

impl RawAudioBuffer {
    /// Construct an `RawAudioBuffer` from a slice and channel count.
    ///
    /// Returns `Err` if `channels` is zero or `buffer.len()` is not divisible
    /// by `channels`. `buffer.len()` must equal `block_size * channels`.
    ///
    /// # SAFETY
    ///
    /// - Caller must guarantee that the buffer passed in lives as long as the underlying `RawAudioBuffer` pointer
    /// - Underlying buffer must not be mutably accessed while the `RawAudioBuffer` exists
    pub unsafe fn new(buffer: &[Sample], channels: usize) -> Result<Self, AudioBufferError> {
        if channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        if !buffer.len().is_multiple_of(channels) {
            return Err(AudioBufferError::LengthChannelMismatch {
                len: buffer.len(),
                channels,
            });
        }

        Ok(Self {
            ptr: NonNull::from(buffer),
            channels,
        })
    }

    pub fn as_slice(&self) -> &[Sample] {
        unsafe { self.ptr.as_ref() }
    }

    pub fn as_f32_slice(&self) -> &[f32] {
        unsafe { core::mem::transmute(self.as_slice()) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Sample] {
        unsafe { self.ptr.as_mut() }
    }

    pub fn as_mut_f32_slice(&mut self) -> &mut [f32] {
        unsafe { core::mem::transmute(self.as_mut_slice()) }
    }
}

impl crate::traits::AudioBuffer for RawAudioBuffer {
    fn block_size(&self) -> usize {
        self.as_slice().len() / self.channels
    }

    fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the samples for channel `c` (0-indexed).
    ///
    /// Returns `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    fn channel(&self, channel: impl Into<Channel>) -> Result<&[Sample], AudioBufferError> {
        let channel = *channel.into();
        if channel >= self.channels {
            return Err(AudioBufferError::ChannelOutOfRange {
                index: channel,
                channels: self.channels,
            });
        }

        let block_size = self.block_size();
        let start = channel * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        // Must not also mutably alias this same data at the same time
        Ok(unsafe {
            core::slice::from_raw_parts((self.ptr.as_ptr() as *const Sample).add(start), block_size)
        })
    }

    fn channels_iter(&self) -> Result<impl Iterator<Item = &[Sample]>, AudioBufferError> {
        if self.channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        Ok(self
            .as_slice()
            .chunks_exact(self.as_slice().len() / self.channels))
    }

    /// Returns the single channel's samples.
    ///
    /// Returns `Err(NotMono)` if `channels != 1`.
    fn mono(&self) -> Result<&[Sample], AudioBufferError> {
        if self.channels != 1 {
            return Err(AudioBufferError::NotMono {
                channels: self.channels,
            });
        }

        let len = { self.ptr.len() };

        // SAFETY: ptr is valid for ptr.len() samples.
        // Must not also mutably alias this same data at the same time
        Ok(unsafe { core::slice::from_raw_parts(self.ptr.as_ptr() as *const Sample, len) })
    }

    fn as_slice(&self) -> &[Sample] {
        unsafe { self.ptr.as_ref() }
    }
}

impl crate::traits::AudioBufferMut for RawAudioBuffer {
    /// Returns the mutable samples for channel `c` (0-indexed).
    ///
    /// Returns `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    fn channel_mut(
        &mut self,
        channel: impl Into<Channel>,
    ) -> Result<&mut [Sample], AudioBufferError> {
        let channel = *channel.into();
        if channel >= self.channels {
            return Err(AudioBufferError::ChannelOutOfRange {
                index: channel,
                channels: self.channels,
            });
        }

        let block_size = self.block_size();
        let start = channel * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        Ok(unsafe {
            core::slice::from_raw_parts_mut(
                (self.ptr.as_ptr() as *mut Sample).add(start),
                block_size,
            )
        })
    }

    /// Returns mutable access to the single channel's samples.
    ///
    /// Returns `Err(NotMono)` if `channels != 1`.
    fn mono_mut(&mut self) -> Result<&mut [Sample], AudioBufferError> {
        if self.channels != 1 {
            return Err(AudioBufferError::NotMono {
                channels: self.channels,
            });
        }

        let len = { self.ptr.len() };

        // SAFETY: ptr is valid for ptr.len() samples.
        Ok(unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut Sample, len) })
    }

    fn channels_iter_mut(
        &mut self,
    ) -> Result<impl Iterator<Item = &mut [Sample]>, AudioBufferError> {
        if self.channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        let len = { self.ptr.len() };
        let channels = self.channels;
        let chunks_len = len / channels;

        Ok(self.as_slice_mut().chunks_exact_mut(chunks_len))
    }

    /// SAFETY:
    /// - self.ptr must point to a valid [Sample] slice
    /// - The slice must live at least as long as &self (guaranteed by PhantomData)
    /// - The memory is properly aligned and initialized
    fn as_slice_mut(&mut self) -> &mut [Sample] {
        unsafe { self.ptr.as_mut() }
    }
}

/// Extracts the raw slice pointer and channel count via the trait interface.
impl<A: AudioBuffer> From<&A> for RawAudioBuffer {
    fn from(audio_buffer: &A) -> Self {
        Self {
            ptr: NonNull::from(audio_buffer.as_slice()),
            channels: audio_buffer.channels(),
        }
    }
}

impl<M: AudioBufferMut> From<&mut M> for RawAudioBuffer {
    fn from(audio_buffer_mut: &mut M) -> Self {
        Self {
            ptr: NonNull::from(audio_buffer_mut.as_slice_mut()),
            channels: audio_buffer_mut.channels(),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use alloc::vec::Vec;

    use crate::test_utils::*;

    use super::*;

    fn samples(values: &[f32]) -> Vec<Sample> {
        values.iter().map(|&v| Sample::from(v)).collect()
    }

    #[test]
    fn option_raw_audio_buffer_is_three_words() {
        assert_eq!(size_of::<Option<RawAudioBuffer>>(), 3 * size_of::<usize>());
    }

    // --- RawAudioBuffer::new ---

    #[test]
    fn new_mono_succeeds() {
        let data = samples(&[1.0, 2.0, 3.0]);
        assert!(unsafe { RawAudioBuffer::new(&data, 1) }.is_ok());
    }

    #[test]
    fn new_stereo_even_length_succeeds() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        assert!(unsafe { RawAudioBuffer::new(&data, 2) }.is_ok());
    }

    #[test]
    fn new_zero_channels_returns_error() {
        let data = samples(&[1.0]);
        assert_eq!(
            unsafe { RawAudioBuffer::new(&data, 0) }.err().unwrap(),
            AudioBufferError::ZeroChannels
        );
    }

    #[test]
    fn new_length_not_divisible_by_channels_returns_error() {
        let data = samples(&[1.0, 2.0, 3.0]); // 3 samples, 2 channels → not divisible
        assert_eq!(
            unsafe { RawAudioBuffer::new(&data, 2) }.err().unwrap(),
            AudioBufferError::LengthChannelMismatch {
                len: 3,
                channels: 2
            }
        );
    }

    #[test]
    fn new_empty_slice_with_one_channel_succeeds() {
        let data: Vec<Sample> = Vec::new();
        assert!(unsafe { RawAudioBuffer::new(&data, 1) }.is_ok());
    }

    // --- Conformance ---

    #[test]
    fn block_size_channels_slice_len_invariant() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = unsafe { RawAudioBuffer::new(&data, 2) }.unwrap();
        test_block_size_channels_slice_len_invariant(&buf);
    }

    #[test]
    fn channel_returns_planar_region() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = unsafe { RawAudioBuffer::new(&data, 2) }.unwrap();
        test_channel_returns_planar_region(&buf);
    }

    #[test]
    fn channel_out_of_range() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = unsafe { RawAudioBuffer::new(&data, 2) }.unwrap();
        test_channel_out_of_range(&buf);
    }

    #[test]
    fn channels_iter_matches_channels() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = unsafe { RawAudioBuffer::new(&data, 2) }.unwrap();
        test_channels_iter_matches_channels(&buf);
    }

    #[test]
    fn mono_single_channel() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = unsafe { RawAudioBuffer::new(&data, 1) }.unwrap();
        test_mono_single_channel(&buf);
    }

    #[test]
    fn mono_multichannel_returns_not_mono() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = unsafe { RawAudioBuffer::new(&data, 2) }.unwrap();
        test_mono_multichannel_returns_not_mono(&buf);
    }
}
