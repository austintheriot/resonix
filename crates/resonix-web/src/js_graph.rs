use resonix_graph::{errors::GraphAddError, implementations::Graph};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
pub struct JsGraph(Graph);

#[wasm_bindgen]
impl JsGraph {
    // pub fn add_audio_node(&mut self, node: JsValue) -> Result<JsValue, GraphAddError> {
    //     todo!()
    // }
    //
    // pub fn connect(
    //     &mut self,
    //     port_a: PortAddress,
    //     port_b: PortAddress,
    // ) -> Result<&mut Self, GraphConnectionError> {
    //     todo!()
    // }
    //
    // /// TODO: represent this in the type system
    // /// `inputs` and `outputs` are indexed by `ExternalConnectionId`
    // pub fn run<A: AudioBuffer, M: AudioBufferMut>(
    //     &mut self,
    //     inputs: &[Option<A>],
    //     outputs: &mut [Option<M>],
    //     current_time: CurrentTime,
    // ) -> Result<(), GraphRunError> {
    //     todo!()
    // }
}
