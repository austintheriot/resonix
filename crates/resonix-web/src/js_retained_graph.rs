use resonix_graph::{primitives::ExternalBufferMappings, traits::Graph};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::JsGraph;

#[wasm_bindgen]
pub struct JsRetainedGraph {
    graph: JsGraph,
    // TODO: store buffers for these connections
    inputs: Vec<Vec<f32>>,
    outputs: Vec<Vec<f32>>,
}

impl JsRetainedGraph {
    // TODO: improve: just testing for now
    pub fn create_storage(
        block_size: usize,
        mappings: &ExternalBufferMappings,
    ) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
        let mut input_buffers: Vec<Option<Vec<f32>>> = Vec::new();
        let mut output_buffers: Vec<Option<Vec<f32>>> = Vec::new();

        for input_mapping_data in mappings.external_inputs() {
            let mut input_buffer = Vec::with_capacity(input_mapping_data.channels * block_size);
            input_buffer.fill(0.0);
            if input_buffers.len() < **input_mapping_data.id + 1 {
                input_buffers.resize(**input_mapping_data.id + 1, None);
            }
            input_buffers[**input_mapping_data.id] = Some(input_buffer);
        }

        for outupt_mapping_data in mappings.external_outputs() {
            let mut output_buffer = Vec::with_capacity(outupt_mapping_data.channels * block_size);

            output_buffer.fill(0.0);
            if input_buffers.len() < **outupt_mapping_data.id + 1 {
                input_buffers.resize(**outupt_mapping_data.id + 1, None);
            }

            output_buffers[**outupt_mapping_data.id] = Some(output_buffer);
        }

        let mut input_buffers: Vec<Vec<f32>> = input_buffers
            .into_iter()
            .map(|maybe_buffer| maybe_buffer.unwrap())
            .collect();
        let mut output_buffers: Vec<Vec<f32>> = output_buffers
            .into_iter()
            .map(|maybe_buffer| maybe_buffer.unwrap())
            .collect();

        (input_buffers, output_buffers)
    }
}

#[wasm_bindgen]
impl JsRetainedGraph {
    pub fn new() -> Self {
        let mut graph = JsGraph::new();

        let (inputs, outputs) =
            Self::create_storage(*graph.block_size(), &graph.external_buffer_mappings());

        Self {
            graph,
            inputs,
            outputs,
        }
    }

    pub fn print_external_buffer_mappings(&mut self) {
        let mappings = self.graph.external_buffer_mappings();
        log::info!("Mappings! {:#?}", mappings);
    }
}
