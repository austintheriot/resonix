use core::{marker::PhantomData, ptr::NonNull};

use crate::primitives::{Channel, Sample};

use super::{AudioBufferError, RawAudioBuffer};

/// Immutable multi-channel audio buffer view, repr(C)-compatible with
/// `RawAudioBuffer` so it can be obtained via a zero-cost transmute.
///
/// Planar layout: channel `c`, sample `i` = `data[c * block_size + i]`.
#[repr(C)]
pub struct AudioBuffer<'a> {
    pub(crate) ptr: NonNull<[Sample]>,
    pub(crate) channels: usize,
    // DO NOT ADD MORE FIELDS HERE WITHOUT CHECKING TRANSMUTE COMPATIBILITY
    _phantom: PhantomData<&'a [Sample]>,
}

// validate transmute safety at compile time for `AudioBuffer`
// byte-representation size of the struct didn't change
const _: () = assert!(
    core::mem::size_of::<Option<RawAudioBuffer>>()
        == core::mem::size_of::<Option<AudioBuffer<'_>>>()
);
// basic check to make sure fields weren't reordered
const _: () = assert!(
    core::mem::offset_of!(RawAudioBuffer, channels)
        == core::mem::offset_of!(AudioBuffer<'_>, channels)
);

impl<'a> AudioBuffer<'a> {
    /// Construct an `AudioBuffer` from a slice and channel count.
    ///
    /// Returns `Err` if `channels` is zero or `buffer.len()` is not divisible
    /// by `channels`. `buffer.len()` must equal `block_size * channels`.
    pub fn new(buffer: &'a [Sample], channels: usize) -> Result<Self, AudioBufferError> {
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

    /// Construct an `AudioBuffer` from a raw pointer and channel count.
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

impl<'a> crate::traits::AudioBuffer for AudioBuffer<'a> {
    fn block_size(&self) -> usize {
        self.ptr.len() / self.channels
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
        Ok(unsafe {
            core::slice::from_raw_parts((self.ptr.as_ptr() as *const Sample).add(start), block_size)
        })
    }

    fn channels_iter(&self) -> Result<impl Iterator<Item = &[Sample]>, AudioBufferError> {
        if self.channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        Ok(self.as_slice().chunks_exact(self.ptr.len() / self.channels))
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

    /// SAFETY:
    /// - self.ptr must point to a valid [Sample] slice
    /// - The slice must live at least as long as &self (guaranteed by PhantomData)
    /// - The memory is properly aligned and initialized
    fn as_slice(&self) -> &[Sample] {
        unsafe { self.ptr.as_ref() }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::traits::AudioBuffer as _;

    use super::*;

    fn samples(values: &[f32]) -> Vec<Sample> {
        values.iter().map(|&v| Sample::from(v)).collect()
    }

    // --- AudioBuffer::new ---

    #[test]
    fn new_mono_succeeds() {
        let data = samples(&[1.0, 2.0, 3.0]);
        assert!(AudioBuffer::new(&data, 1).is_ok());
    }

    #[test]
    fn new_stereo_even_length_succeeds() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        assert!(AudioBuffer::new(&data, 2).is_ok());
    }

    #[test]
    fn new_zero_channels_returns_error() {
        let data = samples(&[1.0]);
        assert_eq!(
            AudioBuffer::new(&data, 0).err().unwrap(),
            AudioBufferError::ZeroChannels
        );
    }

    #[test]
    fn new_length_not_divisible_by_channels_returns_error() {
        let data = samples(&[1.0, 2.0, 3.0]); // 3 samples, 2 channels → not divisible
        assert_eq!(
            AudioBuffer::new(&data, 2).err().unwrap(),
            AudioBufferError::LengthChannelMismatch {
                len: 3,
                channels: 2
            }
        );
    }

    #[test]
    fn new_empty_slice_with_one_channel_succeeds() {
        let data: Vec<Sample> = Vec::new();
        assert!(AudioBuffer::new(&data, 1).is_ok());
    }

    // --- AudioBuffer::channels_iter ---

    #[test]
    fn single_channel() {
        let expected_slice = [0.0, 1.0, 2.0, 3.0];
        let samples = samples(&expected_slice);
        let audio_buffer = AudioBuffer::new(&samples, 1).unwrap();

        let mut channels_iter = audio_buffer.channels_iter().unwrap();

        assert_eq!(channels_iter.next().unwrap(), samples);
        assert_eq!(channels_iter.next(), None);
    }

    #[test]
    fn four_channels() {
        let expected_slice = [0.0, 1.0, 2.0, 3.0];
        let samples = samples(&expected_slice);
        let audio_buffer = AudioBuffer::new(&samples, 4).unwrap();

        let mut channels_iter = audio_buffer.channels_iter().unwrap();

        assert_eq!(channels_iter.next().unwrap(), [samples[0]]);
        assert_eq!(channels_iter.next().unwrap(), [samples[1]]);
        assert_eq!(channels_iter.next().unwrap(), [samples[2]]);
        assert_eq!(channels_iter.next().unwrap(), [samples[3]]);
        assert_eq!(channels_iter.next(), None);
    }
}
