use core::cell::{Ref, RefMut};

use crate::{
    primitives::{BlockSize, BufferPool, ConnectionId, PortId, Sample},
    traits::AudioNode,
};

/// Per-node bridge between the graph's `NodePortMap` + `BufferPool` and a
/// node's `process` call.  Created fresh each frame in `run()`; never stored.
///
/// The key entry point is [`call_process`], which acquires all `RefCell`
/// borrows, builds the plain-slice arrays, and dispatches to the node.
///
/// ## Acquisition order and self-loop safety
///
/// Output buffers are acquired first (exclusive `try_borrow_mut`).  Input
/// buffers are acquired second (shared `try_borrow`).  If a node's output
/// port is wired back to its own input port the input `try_borrow` will fail
/// because the buffer is already exclusively borrowed — that input slot
/// becomes `None` rather than panicking or producing undefined behaviour.
///
/// ## Node removal safety
///
/// This type holds `&'pool BufferPool`, so no buffer can be freed while a
/// context is live.
///
/// [`call_process`]: NodeProcessContext::call_process
pub struct NodeProcessContext<'pool> {
    inputs: &'pool [Option<ConnectionId>],
    outputs: &'pool [Option<ConnectionId>],
    pool: &'pool BufferPool,
    block_size: BlockSize,
}

impl<'pool> NodeProcessContext<'pool> {
    pub fn new(
        inputs: &'pool [Option<ConnectionId>],
        outputs: &'pool [Option<ConnectionId>],
        pool: &'pool BufferPool,
        block_size: BlockSize,
    ) -> Self {
        Self {
            inputs,
            outputs,
            pool,
            block_size,
        }
    }

    /// Acquires `RefCell` borrows for all connected ports, deref's them into
    /// plain slices, and calls `node.process(inputs, outputs, block_size)`.
    ///
    /// All borrows are held for the duration of the `process` call and
    /// released when this method returns.
    ///
    /// ## Acquisition order
    ///
    /// Output borrows (`try_borrow_mut`) are taken first so that self-loop
    /// buffers are claimed before input borrows are attempted.  The
    /// subsequent input `try_borrow` then fails gracefully for those buffers,
    /// yielding `None` rather than a panic.
    ///
    /// ## Stack allocation
    ///
    /// Several fixed-size arrays of `PortId::MAX_PORT_ID` elements are placed
    /// on the stack (~12 KB at the default limit of 256).  Unconnected ports
    /// are `None` and hold no borrow.
    pub fn call_process(
        &self,
        node: &mut dyn AudioNode,
    ) -> Result<(), crate::errors::AudioNodeRunError> {
        // Step 1: Acquire exclusive output borrows first.
        // Self-loop buffers are claimed here; input try_borrow yields None for them.
        let mut output_guards: [Option<RefMut<'pool, [Sample]>>; PortId::MAX_PORT_ID] =
            core::array::from_fn(|i| {
                self.outputs
                    .get(i)?
                    .as_ref()
                    .and_then(|id| self.pool.get(id))
                    .and_then(|c| c.try_borrow_mut().ok())
                    .map(|g| RefMut::map(g, |b| b.as_mut()))
            });

        // Step 2: Extract raw pointers from guards.
        // This decouples the &mut [Sample] lifetime from the per-guard borrow,
        // allowing the slice array to be built after the guard array is complete.
        let output_ptrs: [Option<*mut [Sample]>; PortId::MAX_PORT_ID] = core::array::from_fn(|i| {
            output_guards[i].as_mut().map(|g| &mut **g as *mut [Sample])
        });

        // Step 3: Acquire shared input borrows.
        // Self-loop buffers are already exclusively borrowed above → None here.
        let input_guards: [Option<Ref<'pool, [Sample]>>; PortId::MAX_PORT_ID] =
            core::array::from_fn(|i| {
                self.inputs
                    .get(i)?
                    .as_ref()
                    .and_then(|id| self.pool.get(id))
                    .and_then(|c| c.try_borrow().ok())
                    .map(|g| Ref::map(g, |b| b.as_ref()))
            });

        let inputs: [Option<&[Sample]>; PortId::MAX_PORT_ID] =
            core::array::from_fn(|i| input_guards[i].as_deref());

        // Step 4: Convert raw output pointers to &mut [Sample].
        //
        // SAFETY:
        // - output_guards[i] holds an exclusive borrow of buffer i for the
        //   entire duration of this function; output_ptrs[i] points into that
        //   buffer's allocation.
        // - `output_slices` is declared after `output_guards` and is therefore
        //   dropped first (Rust drops locals in reverse declaration order), so
        //   the &mut [Sample] references never outlive their guards.
        // - No two output slots share a ConnectionId (graph invariant), so no
        //   two elements of output_slices alias the same memory.
        let mut output_slices: [Option<&mut [Sample]>; PortId::MAX_PORT_ID] =
            core::array::from_fn(|i| output_ptrs[i].map(|p| unsafe { &mut *p }));

        node.process(&inputs, &mut output_slices, self.block_size)
    }
}
