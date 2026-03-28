use thiserror::Error;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

use super::AudioNodeRunError;

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Error, Debug)]
pub enum GraphRunError {
    #[error("visit order unexpectedly included the id of a non-node value")]
    VisitOrderIncludedNonNodeValue,
    #[error("node unexpectedly returned an error when processing inputs")]
    NodeReturnedError,
    #[error("audio node encountered error while processing graph")]
    AudioNodeRunError(#[from] AudioNodeRunError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AudioBufferError;
    use std::string::ToString;

    #[test]
    fn visit_order_included_non_node_value_display_is_correct() {
        let error = GraphRunError::VisitOrderIncludedNonNodeValue;
        assert_eq!(
            error.to_string(),
            "visit order unexpectedly included the id of a non-node value"
        );
    }

    #[test]
    fn node_returned_error_display_is_correct() {
        let error = GraphRunError::NodeReturnedError;
        assert_eq!(
            error.to_string(),
            "node unexpectedly returned an error when processing inputs"
        );
    }

    #[test]
    fn audio_node_run_error_display_is_correct() {
        let inner = AudioNodeRunError::TooManyInputs {
            expected: 1,
            found: 2,
        };
        let error = GraphRunError::AudioNodeRunError(inner);
        assert_eq!(
            error.to_string(),
            "audio node encountered error while processing graph"
        );
    }

    #[test]
    fn from_audio_node_run_error_wraps_into_correct_variant() {
        let inner = AudioNodeRunError::AudioBuffer(AudioBufferError::ZeroChannels);
        let error: GraphRunError = inner.into();
        assert!(matches!(error, GraphRunError::AudioNodeRunError(_)));
    }
}
