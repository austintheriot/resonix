use js_sys::{Array, Float32Array};
use resonix_graph::{
    implementations::{Graph, OutputNode, OwnedAudioBuffer, RetainedGraph, SineNode},
    primitives::{AudioNodeCtx, BlockSize, CurrentTime, SampleRate},
    traits::{ModifyGraph as _, RetainedGraph as _},
};
use wasm_bindgen::prelude::wasm_bindgen;

/// Currently the spec specifiec that Web Audio API's
/// AudioWorkletProcessor always delivers 128-sample blocks.
///
/// This is subject to change in the future.
const WEB_BLOCK_SIZE: usize = 128;

/// Thin wrapper over the base `RetainedGraph` for copying
/// web-format-specific inputs/outputs into/out of wasm linear memory.
#[wasm_bindgen]
pub struct JsRetainedGraph(RetainedGraph<Graph>);

impl JsRetainedGraph {
    /// Aligns the web's list of input/output buffer channel with the corresponding portion
    /// of this Graph's retained input/output storage and iterates over them, for easy
    /// copying into/out of Wasm.
    fn iter_aligned_web_and_internal_buffers<F: Fn(Float32Array, &mut [f32])>(
        web_input_output_list: Array<Array<Float32Array>>,
        internal_input_output_list: &mut [Option<OwnedAudioBuffer>],
        process: F,
    ) {
        for (input_output_i, web_input_output) in web_input_output_list.into_iter().enumerate() {
            // Only process inputs/outputs the graph declared; ignore extras from Web Audio.
            let Some(Some(internal_buffer)) = internal_input_output_list.get_mut(input_output_i)
            else {
                break;
            };

            for (channel_i, web_channel_buffer) in web_input_output.into_iter().enumerate() {
                let slice_start = channel_i * WEB_BLOCK_SIZE;
                let slice_end = slice_start + WEB_BLOCK_SIZE;
                let internal_buffer_slice =
                    &mut internal_buffer.as_f32_mut_slice()[slice_start..slice_end];

                process(web_channel_buffer, internal_buffer_slice);
            }
        }
    }

    /// Copy each channel from the JS `inputs` array-of-arrays into the
    /// corresponding contiguous planar buffer in `inputs`.
    ///
    /// Web Audio supplies `inputs[nodeIndex][channelIndex]: Float32Array` while
    /// Resonix expects a single planar `[C0S0..C0SN, C1S0..C1SN, …]` slice.
    /// Extra inputs beyond what the graph declared are ignored.
    fn copy_input_buffer_data_into_wasm(&mut self, inputs: Array<Array<Float32Array>>) {
        Self::iter_aligned_web_and_internal_buffers(
            inputs,
            self.0.get_inputs_mut(),
            |web_buffer, internal_slice| {
                web_buffer.copy_to(internal_slice);
            },
        );
    }

    /// Copy each channel from `outputs` back into the JS `outputs` Float32Arrays,
    /// so the Web Audio pipeline can consume the rendered audio.
    ///
    /// Extra outputs beyond what the graph declared are ignored.
    fn copy_output_buffer_data_out_of_wasm(&mut self, outputs: Array<Array<Float32Array>>) {
        Self::iter_aligned_web_and_internal_buffers(
            outputs,
            self.0.get_outputs_mut(),
            |web_buffer, internal_slice| {
                web_buffer.copy_from(internal_slice);
            },
        );
    }
}

#[wasm_bindgen]
impl JsRetainedGraph {
    // TODO: delete
    #[allow(clippy::new_without_default)]
    pub fn create_test_graph() -> Self {
        let mut graph = Graph::with_block_size(WEB_BLOCK_SIZE);

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

        let retained_graph = RetainedGraph::from(graph);

        Self(retained_graph)
    }

    pub fn run(
        &mut self,
        inputs: Array<Array<Float32Array>>,
        outputs: Array<Array<Float32Array>>,
        current_time: f64,
        sample_rate: u32,
    ) -> Result<bool, String> {
        self.copy_input_buffer_data_into_wasm(inputs);

        let ctx = AudioNodeCtx::builder()
            .current_time(CurrentTime::from(current_time))
            // TODO: bake sample rate & block size at graph instantiation
            .sample_rate(SampleRate::from(sample_rate))
            .block_size(BlockSize::from(WEB_BLOCK_SIZE))
            .build();

        // the raw error is not wasm-compatible
        self.0.run(ctx).map_err(|e| e.to_string())?;

        self.copy_output_buffer_data_out_of_wasm(outputs);

        Ok(true)
    }
}
