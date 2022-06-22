/// This is the default sample rate that is used in the app until state
/// is updated with the true sample rate from the audio context.
pub const FALLBACK_SAMPLE_RATE: u32 = 44100;

pub const MAX_NUM_CHANNELS: usize = 250;

pub const GRAIN_LEN_MIN_IN_MS: usize = 10;

pub const GRAIN_LEN_MAX_IN_MS: usize = 1000;
