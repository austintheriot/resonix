use crate::primitives::ExternalConnectionId;

#[cfg(feature = "js")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Contains per-buffer data for external inputs/outputs
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "js", wasm_bindgen)]
pub struct ExternalBufferMappingData {
    pub id: ExternalConnectionId,
    pub channels: usize,
}
