use core::cell::UnsafeCell;

use alloc::boxed::Box;

use crate::{
    errors::AudioBufferError,
    primitives::{Channel, Sample},
    traits::AudioBuffer,
};

/// A audio buffer that owns its underlying data.
///
/// `data` holds `block_size * channels` samples in planar layout.
/// The slice is wrapped in `UnsafeCell` so that raw pointers derived from
/// `UnsafeCell::get()` carry SharedReadWrite (SRW) provenance under Stacked
/// Borrows, preventing invalidation when downstream refreences hold both an
/// input pointer (downstream node reads) and an output pointer (upstream
/// node writes) to the same buffer simultaneously.
#[derive(Debug)]
pub struct OwnedAudioBuffer {
    pub channels: usize,
    pub data: Box<UnsafeCell<[Sample]>>,
}

impl OwnedAudioBuffer {
    pub fn from_sample_buffer(buffer: Box<[Sample]>, channels: usize) -> Self {
        // SAFETY: `UnsafeCell<[Sample]>` is `repr(transparent)` over `[Sample]`,
        // so `Box<[Sample]>` and `Box<UnsafeCell<[Sample]>>` have identical layouts.
        let data: Box<UnsafeCell<[Sample]>> =
            unsafe { Box::from_raw(Box::into_raw(buffer) as *mut UnsafeCell<[Sample]>) };

        OwnedAudioBuffer { channels, data }
    }

    pub fn from_f32_buffer(buffer: Box<[f32]>, channels: usize) -> Self {
        // SAFETY: `UnsafeCell<[Sample]>` is `repr(transparent)` over `[Sample]`,
        // so `Box<[Sample]>` and `Box<UnsafeCell<[Sample]>>` have identical layouts.
        let data: Box<UnsafeCell<[Sample]>> =
            unsafe { Box::from_raw(Box::into_raw(buffer) as *mut UnsafeCell<[Sample]>) };

        OwnedAudioBuffer { channels, data }
    }

    pub fn as_f32_slice(&self) -> &[f32] {
        unsafe { &*(self.data.get() as *const [f32]) }
    }

    pub fn as_f32_mut_slice(&mut self) -> &mut [f32] {
        unsafe { &mut *(self.data.get() as *mut [f32]) }
    }
}

impl crate::traits::AudioBuffer for OwnedAudioBuffer {
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
            core::slice::from_raw_parts((self.data.get() as *const Sample).add(start), block_size)
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

        let len = { self.data.get().len() };

        // SAFETY: ptr is valid for ptr.len() samples.
        // Must not also mutably alias this same data at the same time
        Ok(unsafe { core::slice::from_raw_parts(self.data.get() as *const Sample, len) })
    }

    fn as_slice(&self) -> &[Sample] {
        unsafe { &*self.data.get() }
    }
}

impl crate::traits::AudioBufferMut for OwnedAudioBuffer {
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
            core::slice::from_raw_parts_mut((self.data.get() as *mut Sample).add(start), block_size)
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

        let len = { self.data.get().len() };

        // SAFETY: ptr is valid for ptr.len() samples.
        Ok(unsafe { core::slice::from_raw_parts_mut(self.data.get() as *mut Sample, len) })
    }

    fn channels_iter_mut(
        &mut self,
    ) -> Result<impl Iterator<Item = &mut [Sample]>, AudioBufferError> {
        if self.channels == 0 {
            return Err(AudioBufferError::ZeroChannels);
        }

        let len = { self.data.get().len() };
        let channels = self.channels;
        let chunks_len = len / channels;

        Ok(self.as_slice_mut().chunks_exact_mut(chunks_len))
    }

    /// SAFETY:
    /// - self.ptr must point to a valid [Sample] slice
    /// - The slice must live at least as long as &self (guaranteed by PhantomData)
    /// - The memory is properly aligned and initialized
    fn as_slice_mut(&mut self) -> &mut [Sample] {
        self.data.get_mut()
    }
}

#[cfg(test)]
mod tests {
    use core::cell::UnsafeCell;

    use alloc::vec::Vec;

    use crate::primitives::Sample;
    use crate::test_utils::*;

    use super::OwnedAudioBuffer;

    fn make_owned(values: &[f32], channels: usize) -> OwnedAudioBuffer {
        let buf: Vec<Sample> = values.iter().map(|&v| Sample::from(v)).collect();
        let data = unsafe {
            alloc::boxed::Box::from_raw(
                alloc::boxed::Box::into_raw(buf.into_boxed_slice()) as *mut UnsafeCell<[Sample]>
            )
        };
        OwnedAudioBuffer { channels, data }
    }

    // --- Conformance  ---

    #[test]
    fn block_size_channels_slice_len_invariant() {
        let buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_block_size_channels_slice_len_invariant(&buf);
    }

    #[test]
    fn channel_returns_planar_region() {
        let buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_channel_returns_planar_region(&buf);
    }

    #[test]
    fn channel_out_of_range() {
        let buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_channel_out_of_range(&buf);
    }

    #[test]
    fn channels_iter_matches_channels() {
        let buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_channels_iter_matches_channels(&buf);
    }

    #[test]
    fn mono_single_channel() {
        let buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 1);
        test_mono_single_channel(&buf);
    }

    #[test]
    fn mono_multichannel_returns_not_mono() {
        let buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_mono_multichannel_returns_not_mono(&buf);
    }

    // --- Conformance (mut) ---

    #[test]
    fn as_slice_mut_len_invariant() {
        let mut buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_as_slice_mut_len_invariant(&mut buf);
    }

    #[test]
    fn channel_mut_writes_correct_region() {
        let mut buf = make_owned(&[0.0, 0.0, 0.0, 0.0], 2);
        test_channel_mut_writes_correct_region(&mut buf);
    }

    #[test]
    fn channel_mut_out_of_range() {
        let mut buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_channel_mut_out_of_range(&mut buf);
    }

    #[test]
    fn channels_iter_mut_matches_channels() {
        let mut buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_channels_iter_mut_matches_channels(&mut buf);
    }

    #[test]
    fn mono_mut_single_channel() {
        let mut buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 1);
        test_mono_mut_single_channel(&mut buf);
    }

    #[test]
    fn mono_mut_multichannel_returns_not_mono() {
        let mut buf = make_owned(&[1.0, 2.0, 3.0, 4.0], 2);
        test_mono_mut_multichannel_returns_not_mono(&mut buf);
    }
}
