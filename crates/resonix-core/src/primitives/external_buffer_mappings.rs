use alloc::boxed::Box;
use alloc::vec::Vec;

#[cfg(feature = "js")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::primitives::external_buffer_mapping_data::ExternalBufferMappingData;

/// Gives outside caller all the details needed to form the external buffers
/// necessary for passing inputs/outputs in to the Graph
#[cfg_attr(feature = "js", wasm_bindgen)]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExternalBufferMappings {
    /// indexed by `ExternalConnectionId`
    pub(crate) external_inputs: Box<[ExternalBufferMappingData]>,
    /// indexed by `ExternalConnectionId`
    pub(crate) external_outputs: Box<[ExternalBufferMappingData]>,
}

impl ExternalBufferMappings {
    pub fn new(
        external_inputs: Box<[ExternalBufferMappingData]>,
        external_outputs: Box<[ExternalBufferMappingData]>,
    ) -> Self {
        Self {
            external_inputs,
            external_outputs,
        }
    }

    pub fn external_inputs(&self) -> &[ExternalBufferMappingData] {
        &self.external_inputs
    }

    pub fn external_outputs(&self) -> &[ExternalBufferMappingData] {
        &self.external_outputs
    }
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl ExternalBufferMappings {
    pub fn external_inputs_into_vec(&self) -> Vec<ExternalBufferMappingData> {
        self.external_inputs.to_vec()
    }

    pub fn external_outputs_into_vec(&self) -> Vec<ExternalBufferMappingData> {
        self.external_outputs.to_vec()
    }
}
