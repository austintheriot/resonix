use core::ptr::NonNull;

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

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use super::*;

    #[test]
    fn option_raw_audio_buffer_is_three_words() {
        assert_eq!(size_of::<Option<RawAudioBuffer>>(), 3 * size_of::<usize>());
    }
}
