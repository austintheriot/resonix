use core::{marker::PhantomData, ptr::NonNull};

use crate::{
    primitives::{Channel, Sample},
    traits::AudioBuffer as _,
};

use crate::errors::AudioBufferError;

use super::RawAudioBuffer;

/// Mutable multi-channel audio buffer view, repr(C)-compatible with
/// `RawAudioBuffer` so it can be obtained via a zero-cost transmute.
#[repr(C)]
pub struct AudioBufferMut<'a> {
    pub(crate) ptr: NonNull<[Sample]>,
    pub(crate) channels: usize,
    // DO NOT ADD MORE FIELDS HERE WITHOUT CHECKING TRANSMUTE COMPATIBILITY
    _phantom: PhantomData<&'a mut [Sample]>,
}

// validate transmute safety at compile time for `AudioBufferMut`
// byte-representation size of the struct didn't change
const _: () = assert!(
    core::mem::size_of::<Option<RawAudioBuffer>>()
        == core::mem::size_of::<Option<AudioBufferMut<'_>>>()
);
// basic check to make sure fields weren't reordered
const _: () = assert!(
    core::mem::offset_of!(RawAudioBuffer, channels)
        == core::mem::offset_of!(AudioBufferMut<'_>, channels)
);

impl<'a> AudioBufferMut<'a> {
    /// Construct an `AudioBufferMut` from a mutable slice and channel count.
    ///
    /// Returns `Err` if `channels` is zero or `buffer.len()` is not divisible
    /// by `channels`. `buffer.len()` must equal `block_size * channels`.
    pub fn new(buffer: &'a mut [Sample], channels: usize) -> Result<Self, AudioBufferError> {
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
            _phantom: PhantomData,
        })
    }

    /// Construct an `AudioBufferMut` from a raw pointer and channel count.
    ///
    /// Returns `Err` if `channels` is zero or `ptr.len()` is not divisible
    /// by `channels`.
    ///
    /// # Safety
    /// `ptr` must be valid and point to at least `ptr.len()` samples that
    /// live for at least `'a`. `ptr.len()` must equal `block_size * channels`.
    pub unsafe fn from_raw(
        ptr: NonNull<[Sample]>,
        channels: usize,
    ) -> Result<Self, AudioBufferError> {
        if channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }
        if !ptr.len().is_multiple_of(channels) {
            return Err(AudioBufferError::LengthChannelMismatch {
                len: ptr.len(),
                channels,
            });
        }
        Ok(Self {
            ptr,
            channels,
            _phantom: PhantomData,
        })
    }
}

impl<'a> crate::traits::AudioBuffer for AudioBufferMut<'a> {
    fn channels(&self) -> usize {
        self.channels
    }

    fn block_size(&self) -> usize {
        self.ptr.len() / self.channels
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
        Ok(unsafe {
            core::slice::from_raw_parts((self.ptr.as_ptr() as *const Sample).add(start), block_size)
        })
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
        // SAFETY: ptr is valid for ptr.len() samples.
        Ok(unsafe {
            core::slice::from_raw_parts(self.ptr.as_ptr() as *const Sample, self.ptr.len())
        })
    }

    fn channels_iter(&self) -> Result<impl Iterator<Item = &[Sample]>, AudioBufferError> {
        if self.channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        let len = self.ptr.len();
        let channels = self.channels;
        let chunks_len = len / channels;

        Ok(self.as_slice().chunks_exact(chunks_len))
    }

    /// SAFETY:
    /// - self.ptr must point to a valid [Sample] slice
    /// - The slice must live at least as long as &self (guaranteed by PhantomData)
    /// - The memory is properly aligned and initialized
    fn as_slice(&self) -> &[Sample] {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a> crate::traits::AudioBufferMut for AudioBufferMut<'a> {
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
        // SAFETY: ptr is valid for ptr.len() samples.
        Ok(unsafe {
            core::slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut Sample, self.ptr.len())
        })
    }

    fn channels_iter_mut(
        &mut self,
    ) -> Result<impl Iterator<Item = &mut [Sample]>, AudioBufferError> {
        if self.channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        let len = self.ptr.len();
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
impl<'a, A: crate::traits::AudioBuffer> From<&'a A> for AudioBufferMut<'a> {
    fn from(audio_buffer: &A) -> Self {
        Self {
            ptr: NonNull::from(audio_buffer.as_slice()),
            channels: audio_buffer.channels(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::errors::AudioBufferError;
    use crate::implementations::AudioBufferMut;
    use crate::primitives::Sample;
    use crate::test_utils::*;

    fn samples(values: &[f32]) -> Vec<Sample> {
        values.iter().map(|&v| Sample::from(v)).collect()
    }

    // --- AudioBufferMut::new ---

    #[test]
    fn new_mut_mono_succeeds() {
        let mut data = samples(&[1.0, 2.0]);
        assert!(AudioBufferMut::new(&mut data, 1).is_ok());
    }

    #[test]
    fn new_mut_zero_channels_returns_error() {
        let mut data = samples(&[1.0]);
        assert_eq!(
            AudioBufferMut::new(&mut data, 0).err().unwrap(),
            AudioBufferError::ZeroChannels
        );
    }

    #[test]
    fn new_mut_length_not_divisible_by_channels_returns_error() {
        let mut data = samples(&[1.0, 2.0, 3.0]);
        assert_eq!(
            AudioBufferMut::new(&mut data, 2).err().unwrap(),
            AudioBufferError::LengthChannelMismatch {
                len: 3,
                channels: 2
            }
        );
    }

    // --- Conformance ---

    #[test]
    fn block_size_channels_slice_len_invariant() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_block_size_channels_slice_len_invariant(&buf);
    }

    #[test]
    fn channel_returns_planar_region() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_channel_returns_planar_region(&buf);
    }

    #[test]
    fn channel_out_of_range() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_channel_out_of_range(&buf);
    }

    #[test]
    fn channels_iter_matches_channels() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_channels_iter_matches_channels(&buf);
    }

    #[test]
    fn mono_single_channel() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 1).unwrap();
        test_mono_single_channel(&buf);
    }

    #[test]
    fn mono_multichannel_returns_not_mono() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_mono_multichannel_returns_not_mono(&buf);
    }

    // --- Conformance (mut) ---

    #[test]
    fn as_slice_mut_len_invariant() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let mut buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_as_slice_mut_len_invariant(&mut buf);
    }

    #[test]
    fn channel_mut_writes_correct_region() {
        let mut data = samples(&[0.0, 0.0, 0.0, 0.0]);
        let mut buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_channel_mut_writes_correct_region(&mut buf);
    }

    #[test]
    fn channel_mut_out_of_range() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let mut buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_channel_mut_out_of_range(&mut buf);
    }

    #[test]
    fn channels_iter_mut_matches_channels() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let mut buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_channels_iter_mut_matches_channels(&mut buf);
    }

    #[test]
    fn mono_mut_single_channel() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let mut buf = AudioBufferMut::new(&mut data, 1).unwrap();
        test_mono_mut_single_channel(&mut buf);
    }

    #[test]
    fn mono_mut_multichannel_returns_not_mono() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let mut buf = AudioBufferMut::new(&mut data, 2).unwrap();
        test_mono_mut_multichannel_returns_not_mono(&mut buf);
    }
}
