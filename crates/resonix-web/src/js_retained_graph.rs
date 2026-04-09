use core::cell::UnsafeCell;
use core::ptr::NonNull;

use js_sys::{Array, Float32Array};
use resonix_graph::{
    implementations::{AudioBuffer, AudioBufferMut, Graph, OutputNode, OwnedAudioBuffer, SineNode},
    primitives::{CurrentTime, ExternalBufferMappingData, ExternalBufferMappings, Sample},
    traits::Graph as _,
};
use wasm_bindgen::{JsCast as _, prelude::wasm_bindgen};

/// Web Audio API's AudioWorkletProcessor always delivers 128-sample blocks.
const WEB_BLOCK_SIZE: usize = 128;

struct JsRetainedGraphStorage {
    input_storage: Vec<Option<OwnedAudioBuffer>>,
    output_storage: Vec<Option<OwnedAudioBuffer>>,
}

#[wasm_bindgen]
pub struct JsRetainedGraph {
    graph: Graph,
    /// Stable, heap-allocated planar buffers for each external input, indexed by
    /// `ExternalConnectionId`. Each `ChannelledBuffer` holds
    /// `channels * WEB_BLOCK_SIZE` samples in planar layout
    /// `[C0S0..C0SN, C1S0..C1SN, …]`.
    ///
    /// The `Box<UnsafeCell<[Sample]>>` inside `ChannelledBuffer` gives pointers
    /// derived from `.get()` SRW (SharedReadWrite) provenance under Stacked
    /// Borrows. This is necessary because `inputs` below holds live
    /// `AudioBuffer<'static>` pointers into these same allocations while the JS
    /// copy helpers also write to them directly — a plain `Box<[Sample]>` would
    /// give those pointers SRO provenance, which gets invalidated on the first
    /// mutable access.
    input_storage: Vec<Option<OwnedAudioBuffer>>,
    /// Same as `input_storage` but for external outputs.
    output_storage: Vec<Option<OwnedAudioBuffer>>,
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

        let input_storage = Self::allocate_channel_buffers(block_size, mappings.external_inputs());
        let output_storage =
            Self::allocate_channel_buffers(block_size, mappings.external_outputs());

        JsRetainedGraphStorage {
            input_storage,
            output_storage,
        }
    }

    fn allocate_channel_buffers(
        block_size: usize,
        mappings: &[ExternalBufferMappingData],
    ) -> Vec<OwnedAudioBuffer> {
        // Size the slot Vec so that id == index (ids are contiguous from 0).
        let capacity = mappings.iter().map(|m| **m.id + 1).max().unwrap_or(0);

        let mut slots: Vec<Option<OwnedAudioBuffer>> = Vec::with_capacity(capacity);
        slots.resize_with(capacity, || None);

        for m in mappings {
            let total_samples = m.channels * block_size;
            let mut buf: Vec<Sample> = Vec::with_capacity(total_samples);
            buf.resize(total_samples, Sample::default());

            // SAFETY: `UnsafeCell<[Sample]>` is `repr(transparent)` over `[Sample]`,
            // so `Box<[Sample]>` and `Box<UnsafeCell<[Sample]>>` have identical
            // layouts. Mirrors the allocation in `Graph::allocate_empty_buffer_for_connection`.
            let data: Box<UnsafeCell<[Sample]>> = unsafe {
                Box::from_raw(Box::into_raw(buf.into_boxed_slice()) as *mut UnsafeCell<[Sample]>)
            };

            slots[**m.id] = Some(OwnedAudioBuffer {
                channels: m.channels,
                data,
            });
        }

        slots.into_iter().map(|s| s.unwrap()).collect()
    }

    /// Copy each channel from the JS `inputs` array-of-arrays into the
    /// corresponding contiguous planar buffer in `input_storage`.
    ///
    /// Web Audio supplies `inputs[nodeIndex][channelIndex]: Float32Array` while
    /// Resonix expects a single planar `[C0S0..C0SN, C1S0..C1SN, …]` slice.
    /// Extra inputs beyond what the graph declared are ignored.
    fn copy_input_buffer_data_into_wasm(&self, inputs: Array) {
        for (input_i, input) in inputs.into_iter().enumerate() {
            let input: Array = input.unchecked_into();

            // Only process inputs the graph declared; ignore extras from Web Audio.
            let Some(Some(buffer)) = self.input_storage.get(input_i) else {
                break;
            };

            for (channel_i, channel) in input.into_iter().enumerate() {
                let channel: Float32Array = channel.unchecked_into();
                let start = channel_i * WEB_BLOCK_SIZE;

                // SAFETY: `buf.data.get()` has SRW provenance (UnsafeCell).
                // `Sample` is `repr(transparent)` over `f32`, identical layout.
                // `buf.data` holds `channels * WEB_BLOCK_SIZE` samples, so
                // `start..start + WEB_BLOCK_SIZE` is in bounds for valid channel
                // indices (the Web Audio worklet guarantees WEB_BLOCK_SIZE frames).
                let dst: &mut [f32] = unsafe {
                    let base = (buffer.data.get() as *mut Sample).add(start);
                    core::slice::from_raw_parts_mut(base as *mut f32, WEB_BLOCK_SIZE)
                };
                channel.copy_to(dst);
            }
        }
    }

    /// Copy each channel from `output_storage` back into the JS `outputs`
    /// Float32Arrays so the Web Audio pipeline can consume the rendered audio.
    /// Extra outputs beyond what the graph declared are ignored.
    fn copy_output_buffer_data_out_of_wasm(&self, outputs: Array) {
        for (output_i, output) in outputs.into_iter().enumerate() {
            let output: Array = output.unchecked_into();
            // Only process outputs the graph declared; ignore extras from Web Audio.
            let Some(Some(buf)) = self.output_storage.get(output_i) else {
                break;
            };

            for (channel_i, channel) in output.into_iter().enumerate() {
                let channel: Float32Array = channel.unchecked_into();
                let start = channel_i * WEB_BLOCK_SIZE;
                // SAFETY: same SRW and repr(transparent) invariants as the input copy.
                let src: &[f32] = unsafe {
                    let base = (buf.data.get() as *const Sample).add(start);
                    core::slice::from_raw_parts(base as *const f32, WEB_BLOCK_SIZE)
                };
                channel.copy_from(src);
            }
        }
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

        let JsRetainedGraphStorage {
            input_storage,
            output_storage,
        } = Self::create_storage(*graph.block_size(), &graph.external_buffer_mappings());

        Self {
            graph,
            input_storage,
            output_storage,
        }
    }

    pub fn print_external_buffer_mappings(&mut self) {
        let mappings = self.graph.external_buffer_mappings();
        log::info!("Mappings! {:#?}", mappings);
    }

    pub fn process(&mut self, inputs: Array, outputs: Array, current_time: f64) -> bool {
        self.copy_input_buffer_data_into_wasm(inputs);

        self.graph
            .run(
                self.inputs.as_slice(),
                self.outputs.as_mut_slice(),
                CurrentTime::from(current_time),
            )
            .unwrap();

        self.copy_output_buffer_data_out_of_wasm(outputs);

        true
    }
}
