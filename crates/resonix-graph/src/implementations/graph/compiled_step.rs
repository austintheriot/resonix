use alloc::boxed::Box;

use crate::{
    implementations::{RawAudioBuffer, graph::ErasedAudioNode},
    primitives::{BlockSize, ExternalConnectionId},
};

/// One entry in the compiled execution plan produced by `Graph::compile`.
///
/// All buffer pointers are extracted once at compile time and reused across `run` calls.
/// External output slots are re-patched from the caller's buffer map each call.
pub(in crate::implementations::graph) struct CompiledStep {
    /// Raw fat pointer into the `Box<dyn AudioNode>` heap allocation.
    /// Stable because moving a `Box` does not move the heap data it points to.
    pub node: *mut dyn ErasedAudioNode,
    /// Input buffer pointers, one per input port (internal + external), indexed by PortId.
    /// Internal ports are backed by pool buffers and set at `connect` time.
    /// External port slots start as `None` and are patched from the caller's buffer map on each `run` call.
    pub input_buffer_ptrs: Box<[Option<RawAudioBuffer>]>,
    /// Output buffer pointers, one per output port (internal + external), indexed by PortId.
    /// Internal ports are backed by pool buffers and set at `connect` time.
    /// External port slots start as `None` and are patched from the caller's buffer map on each `run` call.
    pub output_buffer_ptrs: Box<[Option<RawAudioBuffer>]>,
    /// Indexed by PortId. `Some(ext_id)` marks an external output slot; patched each `run` call.
    pub external_output_slots: Box<[Option<ExternalConnectionId>]>,
    /// Indexed by PortId. `Some(ext_id)` marks an external input slot; patched each `run` call.
    pub external_input_slots: Box<[Option<ExternalConnectionId>]>,
    pub block_size: BlockSize,
}
