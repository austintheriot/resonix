use crate::{errors::AudioBufferError, primitives::Sample, traits::AudioBufferMut};

/// `as_slice_mut().len() == block_size() * channels()`
pub fn test_as_slice_mut_len_invariant(buf: &mut impl AudioBufferMut) {
    let expected_len = buf.block_size() * buf.channels();
    assert_eq!(buf.as_slice_mut().len(), expected_len);
}

/// `channel_mut(c)` modifies the planar region `as_slice()[c*block_size..(c+1)*block_size]`
pub fn test_channel_mut_writes_correct_region(buf: &mut impl AudioBufferMut) {
    let block_size = buf.block_size();
    let channels = buf.channels();
    for channel in 0..channels {
        let sentinel = Sample::from((channel + 1) as f32);
        for sample in buf.channel_mut(channel).unwrap().iter_mut() {
            *sample = sentinel;
        }
    }
    for channel in 0..channels {
        let sentinel = Sample::from((channel + 1) as f32);
        let region = &buf.as_slice()[channel * block_size..(channel + 1) * block_size];
        assert!(region.iter().all(|&s| s == sentinel));
    }
}

/// `channel_mut(channels())` returns `ChannelOutOfRange`
pub fn test_channel_mut_out_of_range(buf: &mut impl AudioBufferMut) {
    let channels = buf.channels();
    assert_eq!(
        buf.channel_mut(channels).unwrap_err(),
        AudioBufferError::ChannelOutOfRange {
            index: channels,
            channels,
        }
    );
}

/// `channels_iter_mut()` yields exactly `channels()` slices of `block_size()` each
pub fn test_channels_iter_mut_matches_channels(buf: &mut impl AudioBufferMut) {
    let block_size = buf.block_size();
    let channels = buf.channels();
    let mut count = 0;
    let iter = buf.channels_iter_mut().unwrap();
    for slice in iter {
        assert_eq!(slice.len(), block_size);
        count += 1;
    }
    assert_eq!(count, channels);
}

/// On a mono buffer: `mono_mut()` covers the full slice
pub fn test_mono_mut_single_channel(buf: &mut impl AudioBufferMut) {
    assert_eq!(buf.channels(), 1, "requires a mono buffer");
    let block_size = buf.block_size();
    let sentinel = Sample::from(42.0f32);
    for sample in buf.mono_mut().unwrap().iter_mut() {
        *sample = sentinel;
    }
    assert_eq!(buf.as_slice().len(), block_size);
    assert!(buf.as_slice().iter().all(|&s| s == sentinel));
}

/// On a multi-channel buffer: `mono_mut()` returns `NotMono`
pub fn test_mono_mut_multichannel_returns_not_mono(buf: &mut impl AudioBufferMut) {
    assert!(buf.channels() > 1, "requires channels > 1");
    let channels = buf.channels();
    assert_eq!(
        buf.mono_mut().unwrap_err(),
        AudioBufferError::NotMono { channels }
    );
}
