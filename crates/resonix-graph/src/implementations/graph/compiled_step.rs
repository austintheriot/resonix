use alloc::boxed::Box;

use crate::{
    implementations::{RawAudioBuffer, graph::ErasedAudioNode},
    primitives::{BlockSize, ConnectionId},
};

/// One entry in the compiled execution plan produced by `Graph::compile`.
///
/// All buffer pointers are extracted once at compile time and reused across `run` calls.
/// External output slots are re-patched from the caller's buffer map each call.
pub(in crate::implementations::graph) struct CompiledStep {
    /// Raw fat pointer into the `Box<dyn AudioNode>` heap allocation.
    /// Stable because moving a `Box` does not move the heap data it points to.
    pub node: *mut dyn ErasedAudioNode,
    /// Input buffer pointers, one per input port, sized to actual port count.
    /// `None` means the port is unconnected or is a self-loop.
    pub input_ptrs: Box<[Option<RawAudioBuffer>]>,
    /// Output buffer pointers, one per output port, sized to actual port count.
    /// Slots for external ports start as `None` and are patched per `run` call.
    pub output_ptrs: Box<[Option<RawAudioBuffer>]>,
    /// Which output slots are external (caller-supplied) and their `ConnectionId`
    /// so they can be looked up in the caller's output map each `run` call.
    pub external_output_slots: Box<[(usize, ConnectionId)]>,
    /// Which input slots are external (caller-supplied) and their `ConnectionId`
    /// so they can be looked up in the caller's input map each `run` call.
    pub external_input_slots: Box<[(usize, ConnectionId)]>,
    pub block_size: BlockSize,
}
