use js_sys::{Array, Float32Array};
use resonix_graph::{
    implementations::{Graph, OutputNode, OwnedAudioBuffer, SineNode},
    primitives::{CurrentTime, ExternalBufferMappingData, ExternalBufferMappings, Sample},
    traits::Graph as _,
};
use wasm_bindgen::prelude::wasm_bindgen;

/// Currently the spec specifiec that Web Audio API's
/// AudioWorkletProcessor always delivers 128-sample blocks.
///
/// This is subject to change in the future.
const WEB_BLOCK_SIZE: usize = 128;

struct JsRetainedGraphStorage {
    inputs: Vec<Option<OwnedAudioBuffer>>,
    outputs: Vec<Option<OwnedAudioBuffer>>,
}

#[wasm_bindgen]
pub struct JsRetainedGraph {
    graph: Graph,
    storage: JsRetainedGraphStorage,
}

impl JsRetainedGraph {
    /// Allocate zeroed, heap-stable planar buffers for every external input and
    /// output declared in `mappings`, then build the pre-baked
    /// `AudioBuffer`/`AudioBufferMut` views that `Graph::run` will consume.
    fn create_storage(
        block_size: usize,
        mappings: &ExternalBufferMappings,
    ) -> JsRetainedGraphStorage {
        log::info!("Creating storage based on mapping: {:?}", mappings);

        let inputs = Self::allocate_audio_buffers(block_size, mappings.external_inputs());
        let outputs = Self::allocate_audio_buffers(block_size, mappings.external_outputs());

        JsRetainedGraphStorage { inputs, outputs }
    }

    fn allocate_audio_buffers(
        block_size: usize,
        mappings: &[ExternalBufferMappingData],
    ) -> Vec<Option<OwnedAudioBuffer>> {
        // Size the slot Vec so that id == index (ids are contiguous from 0).
        let storage_capacity = mappings
            .iter()
            .map(|mapping| **mapping.id + 1)
            .max()
            .unwrap_or(0);

        let mut buffers: Vec<Option<OwnedAudioBuffer>> = Vec::with_capacity(storage_capacity);
        buffers.resize_with(storage_capacity, || None);

        for mapping in mappings {
            let total_samples = mapping.channels * block_size;

            let mut buffer: Vec<Sample> = Vec::with_capacity(total_samples);
            buffer.resize(total_samples, Sample::default());

            buffers[**mapping.id] = Some(OwnedAudioBuffer::from_sample_buffer(
                buffer.into_boxed_slice(),
                mapping.channels,
            ));
        }

        buffers
    }

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
            self.storage.inputs.as_mut_slice(),
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
            self.storage.outputs.as_mut_slice(),
            |web_buffer, internal_slice| {
                web_buffer.copy_from(internal_slice);
            },
        );
    }
}

#[wasm_bindgen]
impl JsRetainedGraph {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
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

        let storage = Self::create_storage(*graph.block_size(), &graph.external_buffer_mappings());

        Self { graph, storage }
    }

    pub fn print_external_buffer_mappings(&mut self) {
        let mappings = self.graph.external_buffer_mappings();
        log::info!("Mappings! {:#?}", mappings);
    }

    pub fn process(
        &mut self,
        inputs: Array<Array<Float32Array>>,
        outputs: Array<Array<Float32Array>>,
        current_time: f64,
    ) -> bool {
        self.copy_input_buffer_data_into_wasm(inputs);

        self.graph
            .run(
                self.storage.inputs.as_slice(),
                self.storage.outputs.as_mut_slice(),
                CurrentTime::from(current_time),
            )
            .unwrap();

        self.copy_output_buffer_data_out_of_wasm(outputs);

        true
    }
}
