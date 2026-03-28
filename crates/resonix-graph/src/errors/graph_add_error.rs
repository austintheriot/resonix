use thiserror::Error;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::errors::common::BufferAlreadyAllocated;

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Error, Debug)]
pub enum GraphAddError {
    #[error(transparent)]
    BufferAlreadyAllocated(#[from] BufferAlreadyAllocated),

    #[error(
        "port IDs must be densely packed starting from 0 within each direction; expected port ID {expected}, found {actual}"
    )]
    SparsePortIds { expected: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn sparse_port_ids_display_shows_expected_and_actual_values() {
        let error = GraphAddError::SparsePortIds {
            expected: 1,
            actual: 3,
        };
        assert_eq!(
            error.to_string(),
            "port IDs must be densely packed starting from 0 within each direction; expected port ID 1, found 3"
        );
    }

    #[test]
    fn from_buffer_already_allocated_wraps_into_correct_variant() {
        let error: GraphAddError = BufferAlreadyAllocated.into();
        assert!(matches!(error, GraphAddError::BufferAlreadyAllocated(_)));
    }

    #[test]
    fn buffer_already_allocated_display_is_transparent() {
        let error: GraphAddError = BufferAlreadyAllocated.into();
        assert_eq!(
            error.to_string(),
            "Internal error: buffer already allocated"
        );
    }
}
