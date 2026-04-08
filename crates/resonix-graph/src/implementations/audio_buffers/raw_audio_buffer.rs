use core::ptr::NonNull;

use crate::primitives::Sample;
use crate::traits::{AudioBuffer, AudioBufferMut};

/// Internal repr(C) buffer handle used inside the compiled execution plan.
///
/// `Option<RawAudioBuffer>` uses the null-pointer niche of `ptr` (the first
/// field), giving it the same size as three `usize`s with no discriminant.
#[repr(C)]
pub struct RawAudioBuffer {
    /// Fat pointer: data pointer + (block_size * channels) as the length.
    pub ptr: NonNull<[Sample]>,
    pub channels: usize,
}

impl RawAudioBuffer {
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

/// Extracts the raw slice pointer and channel count via the trait interface.
/// Layout compatibility is only required between `RawAudioBuffer` and the
/// internal `AudioBuffer<'_>`/`AudioBufferMut<'_>`, guaranteed by `#[repr(C)]`.
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

    use super::*;

    #[test]
    fn option_raw_audio_buffer_is_three_words() {
        assert_eq!(size_of::<Option<RawAudioBuffer>>(), 3 * size_of::<usize>());
    }
}
