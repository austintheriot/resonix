use core::{marker::PhantomData, ptr::NonNull};

use crate::primitives::Sample;

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
    /// Construct an `AudioBuffer` from a raw pointer and channel count.
    ///
    /// # Safety
    /// `ptr` must be valid and point to at least `ptr.len()` samples that
    /// live for at least `'a`. `ptr.len()` must equal `block_size * channels`.
    pub unsafe fn from_raw(ptr: NonNull<[Sample]>, channels: usize) -> Self {
        Self {
            ptr,
            channels,
            _phantom: PhantomData,
        }
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the samples for channel `c` (0-indexed).
    pub fn channel(&self, c: usize) -> &[Sample] {
        let total_len = self.ptr.len();
        let block_size = total_len / self.channels;
        let start = c * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        unsafe {
            core::slice::from_raw_parts((self.ptr.as_ptr() as *const Sample).add(start), block_size)
        }
    }

    /// Returns the single channel's samples. Panics if `channels != 1`.
    pub fn mono(&self) -> &[Sample] {
        assert_eq!(
            self.channels, 1,
            "AudioBuffer::mono() called on non-mono buffer"
        );
        // SAFETY: ptr is valid for ptr.len() samples.
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr() as *const Sample, self.ptr.len()) }
    }
}

impl<'a> AudioBufferMut<'a> {
    /// Construct an `AudioBufferMut` from a raw pointer and channel count.
    ///
    /// # Safety
    /// `ptr` must be valid and point to at least `ptr.len()` samples that
    /// live for at least `'a`. `ptr.len()` must equal `block_size * channels`.
    pub unsafe fn from_raw(ptr: NonNull<[Sample]>, channels: usize) -> Self {
        Self {
            ptr,
            channels,
            _phantom: PhantomData,
        }
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the samples for channel `c` (0-indexed).
    pub fn channel(&self, c: usize) -> &[Sample] {
        let total_len = self.ptr.len();
        let block_size = total_len / self.channels;
        let start = c * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        unsafe {
            core::slice::from_raw_parts((self.ptr.as_ptr() as *const Sample).add(start), block_size)
        }
    }

    /// Returns the mutable samples for channel `c` (0-indexed).
    pub fn channel_mut(&mut self, c: usize) -> &mut [Sample] {
        let total_len = self.ptr.len();
        let block_size = total_len / self.channels;
        let start = c * block_size;
        // SAFETY: ptr is valid for total_len samples; start..start+block_size is in range.
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.ptr.as_ptr() as *mut Sample).add(start),
                block_size,
            )
        }
    }

    /// Returns the single channel's samples. Panics if `channels != 1`.
    pub fn mono(&self) -> &[Sample] {
        assert_eq!(
            self.channels, 1,
            "AudioBufferMut::mono() called on non-mono buffer"
        );
        // SAFETY: ptr is valid for ptr.len() samples.
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr() as *const Sample, self.ptr.len()) }
    }

    /// Returns mutable access to the single channel's samples.
    /// Panics if `channels != 1`.
    pub fn mono_mut(&mut self) -> &mut [Sample] {
        assert_eq!(
            self.channels, 1,
            "AudioBufferMut::mono_mut() called on non-mono buffer"
        );
        // SAFETY: ptr is valid for ptr.len() samples.
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut Sample, self.ptr.len()) }
    }
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use super::*;

    #[test]
    fn option_raw_audio_buffer_is_three_words() {
        assert_eq!(size_of::<Option<RawAudioBuffer>>(), 3 * size_of::<usize>());
    }
}
