use alloc::vec::Vec;

use crate::{
    errors::GraphRunError,
    implementations::OwnedAudioBuffer,
    primitives::{AudioNodeCtx, ExternalBufferMappingData, ExternalBufferMappings, Sample},
};

struct RetainedGraphStorage {
    inputs: Vec<Option<OwnedAudioBuffer>>,
    outputs: Vec<Option<OwnedAudioBuffer>>,
}

pub struct RetainedGraph<G: crate::traits::Graph> {
    graph: G,
    storage: RetainedGraphStorage,
}

impl<G: crate::traits::Graph> From<G> for RetainedGraph<G> {
    fn from(mut graph: G) -> Self {
        let storage = Self::create_storage(*graph.block_size(), &graph.external_buffer_mappings());

        Self { graph, storage }
    }
}

impl<G: crate::traits::Graph> crate::traits::RetainedGraph for RetainedGraph<G> {
    type Buffer = OwnedAudioBuffer;

    fn get_inputs_mut(&mut self) -> &mut [Option<Self::Buffer>] {
        self.storage.inputs.as_mut_slice()
    }

    fn get_outputs_mut(&mut self) -> &mut [Option<Self::Buffer>] {
        self.storage.outputs.as_mut_slice()
    }
}

impl<G: crate::traits::Graph> RetainedGraph<G> {
    pub fn run(&mut self, ctx: AudioNodeCtx) -> Result<(), GraphRunError> {
        self.graph.run(
            self.storage.inputs.as_slice(),
            self.storage.outputs.as_mut_slice(),
            ctx,
        )
    }

    /// Allocate zeroed, heap-stable planar buffers for every external input and
    /// output declared in `mappings`, then build the pre-baked
    /// `AudioBuffer`/`AudioBufferMut` views that `Graph::run` will consume.
    fn create_storage(
        block_size: usize,
        mappings: &ExternalBufferMappings,
    ) -> RetainedGraphStorage {
        let inputs = Self::allocate_audio_buffers(block_size, mappings.external_inputs());
        let outputs = Self::allocate_audio_buffers(block_size, mappings.external_outputs());

        RetainedGraphStorage { inputs, outputs }
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
}

impl<G: crate::traits::Graph> crate::traits::ModifyGraph for RetainedGraph<G> {
    fn add_audio_node<
        P: crate::traits::DescribePorts,
        N: crate::traits::AudioNode + crate::traits::GetPortDescriptors<P> + 'static,
    >(
        &mut self,
        node: N,
    ) -> Result<crate::primitives::NodeHandle<P>, crate::errors::GraphAddError> {
        self.graph.add_audio_node(node)
    }

    fn connect(
        &mut self,
        port_a: crate::primitives::PortAddress,
        port_b: crate::primitives::PortAddress,
    ) -> Result<&mut Self, crate::errors::GraphConnectionError> {
        match self.graph.connect(port_a, port_b) {
            Err(error) => Err(error),
            Ok(_) => Ok(self),
        }
    }

    fn block_size(&self) -> crate::primitives::BlockSize {
        self.graph.block_size()
    }

    fn external_buffer_mappings(&mut self) -> ExternalBufferMappings {
        self.graph.external_buffer_mappings()
    }
}
