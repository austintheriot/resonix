use core::{marker::PhantomData, ptr::NonNull};

use crate::primitives::Sample;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioBufferError {
    ZeroChannels,
    LengthChannelMismatch { len: usize, channels: usize },
    ChannelOutOfRange { index: usize, channels: usize },
    NotMono { channels: usize },
}

/// Internal repr(C) buffer handle used inside the compiled execution plan.
///
/// `Option<RawAudioBuffer>` uses the null-pointer niche of `ptr` (the first
/// field), giving it the same size as three `usize`s with no discriminant.
#[repr(C)]
pub(crate) struct RawAudioBuffer {
    /// Fat pointer: data pointer + (block_size * channels) as the length.
    pub ptr: NonNull<[Sample]>,
    pub channels: usize,
}

/// Immutable multi-channel audio buffer view, repr(C)-compatible with
/// `RawAudioBuffer` so it can be obtained via a zero-cost transmute.
///
/// Planar layout: channel `c`, sample `i` = `data[c * block_size + i]`.
#[repr(C)]
pub struct AudioBuffer<'a> {
    pub(crate) ptr: NonNull<[Sample]>,
    pub(crate) channels: usize,
    _phantom: PhantomData<&'a [Sample]>,
}

/// Mutable multi-channel audio buffer view, repr(C)-compatible with
/// `RawAudioBuffer` so it can be obtained via a zero-cost transmute.
#[repr(C)]
pub struct AudioBufferMut<'a> {
    pub(crate) ptr: NonNull<[Sample]>,
    pub(crate) channels: usize,
    _phantom: PhantomData<&'a mut [Sample]>,
}

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

    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the samples for channel `c` (0-indexed).
    ///
    /// Returns `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    pub fn channel(&self, c: usize) -> Result<&[Sample], AudioBufferError> {
        if c >= self.channels {
            return Err(AudioBufferError::ChannelOutOfRange {
                index: c,
                channels: self.channels,
            });
        }
        let total_len = self.ptr.len();
        let block_size = total_len / self.channels;
        let start = c * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        Ok(unsafe {
            core::slice::from_raw_parts((self.ptr.as_ptr() as *const Sample).add(start), block_size)
        })
    }

    /// Returns the single channel's samples.
    ///
    /// Returns `Err(NotMono)` if `channels != 1`.
    pub fn mono(&self) -> Result<&[Sample], AudioBufferError> {
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
}

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

    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the samples for channel `c` (0-indexed).
    ///
    /// Returns `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    pub fn channel(&self, c: usize) -> Result<&[Sample], AudioBufferError> {
        if c >= self.channels {
            return Err(AudioBufferError::ChannelOutOfRange {
                index: c,
                channels: self.channels,
            });
        }
        let total_len = self.ptr.len();
        let block_size = total_len / self.channels;
        let start = c * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        Ok(unsafe {
            core::slice::from_raw_parts((self.ptr.as_ptr() as *const Sample).add(start), block_size)
        })
    }

    /// Returns the mutable samples for channel `c` (0-indexed).
    ///
    /// Returns `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    pub fn channel_mut(&mut self, c: usize) -> Result<&mut [Sample], AudioBufferError> {
        if c >= self.channels {
            return Err(AudioBufferError::ChannelOutOfRange {
                index: c,
                channels: self.channels,
            });
        }
        let total_len = self.ptr.len();
        let block_size = total_len / self.channels;
        let start = c * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        Ok(unsafe {
            core::slice::from_raw_parts_mut(
                (self.ptr.as_ptr() as *mut Sample).add(start),
                block_size,
            )
        })
    }

    /// Returns the single channel's samples.
    ///
    /// Returns `Err(NotMono)` if `channels != 1`.
    pub fn mono(&self) -> Result<&[Sample], AudioBufferError> {
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

    /// Returns mutable access to the single channel's samples.
    ///
    /// Returns `Err(NotMono)` if `channels != 1`.
    pub fn mono_mut(&mut self) -> Result<&mut [Sample], AudioBufferError> {
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
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::mem::size_of;

    use super::*;

    fn samples(values: &[f32]) -> Vec<Sample> {
        values.iter().map(|&v| Sample::from(v)).collect()
    }

    #[test]
    fn option_raw_audio_buffer_is_three_words() {
        assert_eq!(size_of::<Option<RawAudioBuffer>>(), 3 * size_of::<usize>());
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

    // --- mono / mono_mut ---

    #[test]
    fn mono_on_single_channel_buffer_returns_all_samples() {
        let data = samples(&[1.0, 2.0, 3.0]);
        let buf = AudioBuffer::new(&data, 1).unwrap();
        assert_eq!(buf.mono().unwrap(), data.as_slice());
    }

    #[test]
    fn mono_on_multi_channel_buffer_returns_error() {
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBuffer::new(&data, 2).unwrap();
        assert_eq!(
            buf.mono().err().unwrap(),
            AudioBufferError::NotMono { channels: 2 }
        );
    }

    #[test]
    fn mono_mut_on_single_channel_buffer_allows_writing() {
        let mut data = samples(&[0.0, 0.0]);
        {
            let mut buf = AudioBufferMut::new(&mut data, 1).unwrap();
            for s in buf.mono_mut().unwrap().iter_mut() {
                *s = Sample::from(7.0f32);
            }
        }
        assert_eq!(data, samples(&[7.0, 7.0]));
    }

    #[test]
    fn mono_mut_on_multi_channel_buffer_returns_error() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let mut buf = AudioBufferMut::new(&mut data, 2).unwrap();
        assert_eq!(
            buf.mono_mut().err().unwrap(),
            AudioBufferError::NotMono { channels: 2 }
        );
    }

    // --- channel / channel_mut ---

    #[test]
    fn channel_returns_correct_planar_slice() {
        // Stereo, block_size=2: [L0, L1, R0, R1]
        let data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBuffer::new(&data, 2).unwrap();
        assert_eq!(buf.channel(0).unwrap(), samples(&[1.0, 2.0]).as_slice());
        assert_eq!(buf.channel(1).unwrap(), samples(&[3.0, 4.0]).as_slice());
    }

    #[test]
    fn channel_out_of_range_returns_error() {
        let data = samples(&[1.0, 2.0]);
        let buf = AudioBuffer::new(&data, 1).unwrap();
        assert_eq!(
            buf.channel(1).err().unwrap(),
            AudioBufferError::ChannelOutOfRange {
                index: 1,
                channels: 1
            }
        );
    }

    #[test]
    fn channel_mut_writes_correct_planar_slice() {
        let mut data = samples(&[0.0, 0.0, 0.0, 0.0]);
        {
            let mut buf = AudioBufferMut::new(&mut data, 2).unwrap();
            for s in buf.channel_mut(1).unwrap().iter_mut() {
                *s = Sample::from(9.0f32);
            }
        }
        assert_eq!(data, samples(&[0.0, 0.0, 9.0, 9.0]));
    }

    #[test]
    fn channel_mut_out_of_range_returns_error() {
        let mut data = samples(&[1.0, 2.0]);
        let mut buf = AudioBufferMut::new(&mut data, 1).unwrap();
        assert_eq!(
            buf.channel_mut(1).err().unwrap(),
            AudioBufferError::ChannelOutOfRange {
                index: 1,
                channels: 1
            }
        );
    }
}
