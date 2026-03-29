use resonix_graph::traits::Graph;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::JsGraph;

#[wasm_bindgen]
pub struct JsRetainedGraph {
    graph: JsGraph,
    // TODO: store buffers for these connections
    //inputs: Vec<Vec<f32>>,
    //outputs: Vec<Vec<f32>>,
}

#[wasm_bindgen]
impl JsRetainedGraph {
    pub fn new() -> Self {
        Self {
            graph: JsGraph::new(),
        }
    }

    pub fn print_external_buffer_mappings(&mut self) {
        let mappings = self.graph.external_buffer_mappings();
        log::info!("Mappings! {:#?}", mappings);
    }
}
