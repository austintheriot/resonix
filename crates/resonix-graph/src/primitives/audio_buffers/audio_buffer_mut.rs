use core::{marker::PhantomData, ptr::NonNull};

use crate::primitives::{Channel, Sample};

use super::{AudioBufferError, RawAudioBuffer};

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

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn block_size(&self) -> usize {
        self.ptr.len() / self.channels
    }

    /// Returns the samples for channel `c` (0-indexed).
    ///
    /// Returns `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    pub fn channel(&self, channel: impl Into<Channel>) -> Result<&[Sample], AudioBufferError> {
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

    /// Returns the mutable samples for channel `c` (0-indexed).
    ///
    /// Returns `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    pub fn channel_mut(
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

    /// SAFETY:
    /// - self.ptr must point to a valid [Sample] slice
    /// - The slice must live at least as long as &self (guaranteed by PhantomData)
    /// - The memory is properly aligned and initialized
    fn as_slice(&self) -> &[Sample] {
        unsafe { self.ptr.as_ref() }
    }

    /// SAFETY:
    /// - self.ptr must point to a valid [Sample] slice
    /// - The slice must live at least as long as &self (guaranteed by PhantomData)
    /// - The memory is properly aligned and initialized
    fn as_slice_mut(&mut self) -> &mut [Sample] {
        unsafe { self.ptr.as_mut() }
    }

    pub fn channels_iter(&self) -> Result<impl Iterator<Item = &[Sample]>, AudioBufferError> {
        if self.channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        let len = self.ptr.len();
        let channels = self.channels;
        let chunks_len = len / channels;

        Ok(self.as_slice().chunks_exact(chunks_len))
    }

    pub fn channels_iter_mut(
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
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::primitives::{AudioBufferError, AudioBufferMut, Sample};

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

    // --- mono / mono_mut ---

    #[test]
    fn mono_on_single_channel_buffer_returns_all_samples() {
        let mut data = samples(&[1.0, 2.0, 3.0]);
        let expected_data = data.clone();
        let buf = AudioBufferMut::new(&mut data, 1).unwrap();
        assert_eq!(buf.mono().unwrap(), expected_data.as_slice());
    }

    #[test]
    fn mono_on_multi_channel_buffer_returns_error() {
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 2).unwrap();
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
        let mut data = samples(&[1.0, 2.0, 3.0, 4.0]);
        let buf = AudioBufferMut::new(&mut data, 2).unwrap();
        assert_eq!(buf.channel(0).unwrap(), samples(&[1.0, 2.0]).as_slice());
        assert_eq!(buf.channel(1).unwrap(), samples(&[3.0, 4.0]).as_slice());
    }

    #[test]
    fn channel_out_of_range_returns_error() {
        let mut data = samples(&[1.0, 2.0]);
        let buf = AudioBufferMut::new(&mut data, 1).unwrap();
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

    // --- AudioBufferMut::channels_iter ---

    #[test]
    fn single_channel() {
        let expected_slice = [0.0, 1.0, 2.0, 3.0];
        let mut samples = samples(&expected_slice);
        let expected_samples: Vec<Sample> = samples.to_vec();
        let audio_buffer_mut = AudioBufferMut::new(&mut samples, 1).unwrap();

        let mut channels_iter = audio_buffer_mut.channels_iter().unwrap();

        assert_eq!(channels_iter.next().unwrap(), expected_samples);
        assert_eq!(channels_iter.next(), None);
    }

    #[test]
    fn four_channels() {
        let expected_slice = [0.0, 1.0, 2.0, 3.0];
        let mut samples = samples(&expected_slice);
        let expected_samples: Vec<Sample> = samples.to_vec();
        let audio_buffer_mut = AudioBufferMut::new(&mut samples, 4).unwrap();

        let mut channels_iter = audio_buffer_mut.channels_iter().unwrap();

        assert_eq!(channels_iter.next().unwrap(), [expected_samples[0]]);
        assert_eq!(channels_iter.next().unwrap(), [expected_samples[1]]);
        assert_eq!(channels_iter.next().unwrap(), [expected_samples[2]]);
        assert_eq!(channels_iter.next().unwrap(), [expected_samples[3]]);
        assert_eq!(channels_iter.next(), None);
    }

    // --- AudioBufferMut::channels_iter_mut ---

    #[test]
    fn single_channel_mut() {
        let expected_slice = [0.0, 1.0, 2.0, 3.0];
        let mut samples = samples(&expected_slice);
        let expected_samples: Vec<Sample> = samples.clone().into_iter().collect();
        let mut audio_buffer_mut = AudioBufferMut::new(&mut samples, 1).unwrap();

        let mut channels_iter = audio_buffer_mut.channels_iter_mut().unwrap();

        assert_eq!(channels_iter.next().unwrap(), expected_samples);
        assert_eq!(channels_iter.next(), None);
    }

    #[test]
    fn four_channels_mut() {
        let expected_slice = [0.0, 1.0, 2.0, 3.0];
        let mut samples = samples(&expected_slice);
        let expected_samples: Vec<Sample> = samples.to_vec();
        let mut audio_buffer_mut = AudioBufferMut::new(&mut samples, 4).unwrap();

        let mut channels_iter = audio_buffer_mut.channels_iter_mut().unwrap();

        assert_eq!(channels_iter.next().unwrap(), [expected_samples[0]]);
        assert_eq!(channels_iter.next().unwrap(), [expected_samples[1]]);
        assert_eq!(channels_iter.next().unwrap(), [expected_samples[2]]);
        assert_eq!(channels_iter.next().unwrap(), [expected_samples[3]]);
        assert_eq!(channels_iter.next(), None);
    }
}
