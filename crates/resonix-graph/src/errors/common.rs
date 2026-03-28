use thiserror::Error;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Debug, Error)]
#[error("Internal error: buffer already allocated")]
pub struct BufferAlreadyAllocated;

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn display_message_matches_expected_text() {
        let error = BufferAlreadyAllocated;
        assert_eq!(
            error.to_string(),
            "Internal error: buffer already allocated"
        );
    }
}
