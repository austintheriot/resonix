use crate::{errors::AudioBufferError, traits::AudioBuffer};

/// `block_size() * channels() == as_slice().len()`
pub fn test_block_size_channels_slice_len_invariant(buf: &impl AudioBuffer) {
    assert_eq!(buf.block_size() * buf.channels(), buf.as_slice().len());
}

/// `channel(c)` returns the planar region `as_slice()[c*bs..(c+1)*bs]`
pub fn test_channel_returns_planar_region(buf: &impl AudioBuffer) {
    let block_size = buf.block_size();
    for channel in 0..buf.channels() {
        let expected = &buf.as_slice()[channel * block_size..(channel + 1) * block_size];
        assert_eq!(buf.channel(channel).unwrap(), expected);
    }
}

/// `channel(channels())` returns `ChannelOutOfRange`
pub fn test_channel_out_of_range(buf: &impl AudioBuffer) {
    let out_of_range = buf.channels();
    assert_eq!(
        buf.channel(out_of_range).unwrap_err(),
        AudioBufferError::ChannelOutOfRange {
            index: out_of_range,
            channels: buf.channels(),
        }
    );
}

/// `channels_iter()` yields exactly `channels()` items, each matching `channel(c)`
pub fn test_channels_iter_matches_channels(buf: &impl AudioBuffer) {
    let mut iter = buf.channels_iter().unwrap();
    for channel in 0..buf.channels() {
        let expected = buf.channel(channel).unwrap();
        assert_eq!(iter.next().unwrap(), expected);
    }
    assert!(iter.next().is_none());
}

/// On a mono buffer: `mono()` returns all samples (`== as_slice()`)
pub fn test_mono_single_channel(buf: &impl AudioBuffer) {
    assert_eq!(buf.channels(), 1, "requires a mono buffer");
    assert_eq!(buf.mono().unwrap(), buf.as_slice());
}

/// On a multi-channel buffer: `mono()` returns `NotMono`
pub fn test_mono_multichannel_returns_not_mono(buf: &impl AudioBuffer) {
    assert!(buf.channels() > 1, "requires channels > 1");
    assert_eq!(
        buf.mono().unwrap_err(),
        AudioBufferError::NotMono {
            channels: buf.channels(),
        }
    );
}
