use js_sys::Array;
use resonix_graph::{implementations::Graph, primitives::PortAddress, traits::Graph as _};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
pub struct JsGraph(Graph);

#[wasm_bindgen]
impl JsGraph {
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
