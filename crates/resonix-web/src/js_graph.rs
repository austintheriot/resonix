use std::ops::{Deref, DerefMut};

use js_sys::Array;
use resonix_graph::{
    implementations::{Graph, OutputNode, SineNode},
    primitives::PortAddress,
    traits::Graph as _,
};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
pub struct JsGraph(Graph);

#[wasm_bindgen]
impl JsGraph {
    pub fn new() -> Self {
        let web_block_size = 128;
        let mut graph = Graph::with_block_size(web_block_size);

        // TODO: remove this. Pre-initializing is just for testing on web
        let sine_node = SineNode::new_with_frequency(&mut graph, 440.0);
        let output_node = OutputNode::new(&mut graph);

        let sine_node_handle = graph.add_audio_node(sine_node).unwrap();
        let output_node_handle = graph.add_audio_node(output_node).unwrap();

        graph
            .connect(
                sine_node_handle.output_port_address(),
                output_node_handle.input_port_address(),
            )
            .unwrap();

        // TODO: enable dynamic block size,
        // For now, web platform is locked to `128` block size
        Self(graph)
    }

    pub fn add_audio_node(&mut self, node: JsValue) {
        log::info!("Running from Wasm! graph.add_audio_node. Args: {:?} ", node);
        todo!();
    }

    pub fn connect_audio(&mut self, port_a: PortAddress, port_b: PortAddress) {
        self.0.connect(port_a, port_b).unwrap();
    }

    /// TODO: represent this in the type system
    /// `inputs` and `outputs` are indexed by `ExternalConnectionId`
    pub fn run(&mut self, inputs: Array, outputs: Array, current_time: f64) {
        log::info!(
            "Running from Wasm! graph.run. Args: {:?} {:?} {:?}",
            inputs,
            outputs,
            current_time
        );
        todo!();
    }
}

impl Deref for JsGraph {
    type Target = Graph;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JsGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
