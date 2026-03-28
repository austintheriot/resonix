#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioBufferError {
    ZeroChannels,
    LengthChannelMismatch { len: usize, channels: usize },
    ChannelOutOfRange { index: usize, channels: usize },
    NotMono { channels: usize },
}
