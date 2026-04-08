use js_sys::{Array, Float32Array};
use resonix_graph::{
    implementations::{AudioBuffer, AudioBufferMut, Graph, OutputNode, SineNode},
    primitives::{BufferPool, CurrentTime, ExternalBufferMappings},
    traits::Graph as _,
};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct JsRetainedGraph {
    graph: Graph,
    // persistant storage for inputs/outupts
    buffer_pool: BufferPool,
    // pointers into the buffer pool for passing into `run`
    inputs: Vec<Option<AudioBuffer<'static>>>,
    outputs: Vec<Option<AudioBufferMut<'static>>>,
}

impl JsRetainedGraph {
    // TODO: improve: just testing for now
    pub fn create_storage(
        block_size: usize,
        mappings: &ExternalBufferMappings,
    ) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
        log::info!("Creating storage based on mapping: {:?}", mappings);

        // TODO: replace with channeld buffers in the buffer_pool
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

        for output_mapping_data in mappings.external_outputs() {
            let mut output_buffer = Vec::with_capacity(output_mapping_data.channels * block_size);

            output_buffer.fill(0.0);
            if output_buffers.len() < **output_mapping_data.id + 1 {
                output_buffers.resize(**output_mapping_data.id + 1, None);
            }

            output_buffers[**output_mapping_data.id] = Some(output_buffer);
        }

        let input_buffers: Vec<Vec<f32>> = input_buffers
            .into_iter()
            .map(|maybe_buffer| maybe_buffer.unwrap())
            .collect();
        let output_buffers: Vec<Vec<f32>> = output_buffers
            .into_iter()
            .map(|maybe_buffer| maybe_buffer.unwrap())
            .collect();

        // TODO: create `inputs` and `outputs` as pointers into the BufferPool
        // SAFETY: assumes that the `BufferPool` channeled buffers are immutable
        // after creation

        (input_buffers, output_buffers)
    }

    fn copy_input_buffer_data_into_wasm(&mut self, inputs: Array<Array<Float32Array>>) {
        log::info!("Copying input data into internal buffers");

        inputs.into_iter().enumerate().for_each(|(input_i, input)| {
            input
                .into_iter()
                .enumerate()
                .for_each(|(channel_i, channel)| {
                    // TODO: replace with lookup into buffer_pool
                    let Some(backing_buffer) = self.inputs.get_mut(input_i) else {
                        return;
                    };

                    // in Resonix, multi-channel audio buffers are stored as single,
                    // contiguous buffer of data, but web stores different channels
                    // as separate `Float32Array`s, so we must copy them one-by-one
                    // TODO: move this into a separate const / clean up arithmetic logic
                    const WEB_BLOCK_SIZE: usize = 128;
                    let backing_buffer_starting_i = channel_i * WEB_BLOCK_SIZE;
                    let backing_buffer_ending_i = backing_buffer_starting_i + WEB_BLOCK_SIZE;
                    let backing_buffer_channel_slice =
                        &mut backing_buffer[backing_buffer_starting_i..backing_buffer_ending_i];

                    channel.copy_to(backing_buffer_channel_slice);
                });
        });
    }

    fn copy_output_buffer_data_out_of_wasm(&mut self, outputs: Array<Array<Float32Array>>) {
        log::info!("Copying output data out of external buffers");

        outputs
            .into_iter()
            .enumerate()
            .for_each(|(output_i, ouput)| {
                ouput
                    .into_iter()
                    .enumerate()
                    .for_each(|(channel_i, channel)| {
                        // TODO: replace with lookup into buffer_pool
                        let Some(backing_buffer) = self.inputs.get_mut(output_i) else {
                            return;
                        };

                        // in Resonix, multi-channel audio buffers are stored as single,
                        // contiguous buffer of data, but web stores different channels
                        // as separate `Float32Array`s, so we must copy them one-by-one
                        // TODO: move this into a separate const / clean up arithmetic logic
                        const WEB_BLOCK_SIZE: usize = 128;
                        let backing_buffer_starting_i = channel_i * WEB_BLOCK_SIZE;
                        let backing_buffer_ending_i = backing_buffer_starting_i + WEB_BLOCK_SIZE;
                        let backing_buffer_channel_slice =
                            &mut backing_buffer[backing_buffer_starting_i..backing_buffer_ending_i];

                        channel.copy_from(backing_buffer_channel_slice);
                    });
            });
    }
}

#[wasm_bindgen]
impl JsRetainedGraph {
    pub fn new() -> Self {
        // TODO: move to const
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

    pub fn process(
        &mut self,
        inputs: Array<Array<Float32Array>>,
        outputs: Array<Array<Float32Array>>,
        current_time: f64,
    ) -> bool {
        log::info!(
            "Calling process from `JsRetainedGraph` \nInputs = {:?} \nOutputs = {:?}",
            inputs,
            outputs
        );

        self.copy_input_buffer_data_into_wasm(inputs);

        self.graph.run(
            self.inputs.as_slice(),
            self.outputs.as_slice(),
            CurrentTime::from(current_time),
        );

        self.copy_output_buffer_data_out_of_wasm(outputs);

        true
    }
}
