use alloc::{boxed::Box, vec::Vec};

use crate::primitives::{BlockSize, PortId, Sample};

/// Pre-allocated per-node context for zero-allocation DSP processing.
///
/// The `Box`ed arrays are allocated once when a node is added to the graph and
/// reused across every `run()` call. Raw pointers to the actual sample buffers
/// are written into the arrays during `connect()` and `add()` (topology changes)
/// and remain valid for the lifetime of the graph's [`BufferPool`].
///
/// # Safety invariants
///
/// - Every raw pointer stored here either is null (unconnected port) or points
///   to the heap-allocated interior of a `Box<[Sample]>` owned by `BufferPool`.
/// - The graph processes nodes sequentially in topological order, so no two
///   nodes ever access the same buffer simultaneously.
/// - Callers of [`output`] must not invoke it with the same `port_id` more than
///   once per `process` call; doing so would create aliased `&mut` references.
///
/// [`BufferPool`]: crate::primitives::BufferPool
/// [`output`]: NodeProcessContext::output
pub struct NodeProcessContext {
    /// Input buffer raw pointers indexed by port_id.
    /// A null data pointer means the port is not connected.
    inputs: Box<[*const [Sample]]>,
    /// Output buffer raw pointers indexed by port_id.
    /// A null data pointer means the port is not connected.
    outputs: Box<[*mut [Sample]]>,
    block_size: BlockSize,
}

// SAFETY: The raw pointers point into heap-allocated Boxes owned by the graph's
// BufferPool. The graph guarantees exclusive access during processing.
unsafe impl Send for NodeProcessContext {}
unsafe impl Sync for NodeProcessContext {}

impl NodeProcessContext {
    /// Allocates the context arrays.  Called once per node when it is added to
    /// the graph; never called during `run()`.
    pub fn new(num_input_slots: usize, num_output_slots: usize, block_size: BlockSize) -> Self {
        let null_in =
            core::ptr::slice_from_raw_parts(core::ptr::null::<Sample>(), 0) as *const [Sample];
        let null_out = core::ptr::slice_from_raw_parts_mut(core::ptr::null_mut::<Sample>(), 0)
            as *mut [Sample];
        let mut inputs_vec = Vec::with_capacity(num_input_slots);
        inputs_vec.resize(num_input_slots, null_in);
        let mut outputs_vec = Vec::with_capacity(num_output_slots);
        outputs_vec.resize(num_output_slots, null_out);
        Self {
            inputs: inputs_vec.into_boxed_slice(),
            outputs: outputs_vec.into_boxed_slice(),
            block_size,
        }
    }

    /// Registers an input buffer pointer for `port_id`.
    ///
    /// Called during graph topology changes (`connect` / `add`), not during `run`.
    pub fn set_input_ptr(&mut self, port_id: PortId, ptr: *const [Sample]) {
        if let Some(slot) = self.inputs.get_mut(**port_id) {
            *slot = ptr;
        }
    }

    /// Registers an output buffer pointer for `port_id`.
    ///
    /// Called during graph topology changes (`connect` / `add`), not during `run`.
    pub fn set_output_ptr(&mut self, port_id: PortId, ptr: *mut [Sample]) {
        if let Some(slot) = self.outputs.get_mut(**port_id) {
            *slot = ptr;
        }
    }

    /// Returns an immutable view of the input buffer for `port_id`, or `None`
    /// if the port is unconnected.
    pub fn input(&self, port_id: impl Into<PortId>) -> Option<&[Sample]> {
        let ptr = *self.inputs.get(**port_id.into())?;
        if ptr.is_null() {
            None
        } else {
            // SAFETY: ptr was set from a valid heap-allocated Box<[Sample]> owned by
            // the graph's BufferPool.  The graph guarantees no concurrent mutable access
            // while we hold this shared reference.
            Some(unsafe { &*ptr })
        }
    }

    /// Returns a mutable view of the output buffer for `port_id`, or `None`
    /// if the port is unconnected.
    ///
    /// # Safety contract
    ///
    /// Do **not** call this with the same `port_id` more than once within a
    /// single `process` invocation.  Doing so would produce aliased `&mut`
    /// references pointing to the same memory, which is undefined behaviour.
    pub fn output(&self, port_id: impl Into<PortId>) -> Option<&mut [Sample]> {
        let ptr = *self.outputs.get(**port_id.into())?;
        if ptr.is_null() {
            None
        } else {
            // SAFETY: ptr was set from a valid heap-allocated Box<[Sample]> owned by
            // the graph's BufferPool.  The graph processes nodes sequentially, so no
            // other node holds a reference to this buffer at the same time.
            Some(unsafe { &mut *ptr })
        }
    }

    pub fn block_size(&self) -> BlockSize {
        self.block_size
    }
}
