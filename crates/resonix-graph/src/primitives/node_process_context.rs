use core::cell::{Ref, RefMut};

use crate::{
    primitives::{BlockSize, BufferPool, ConnectionId, PortId, Sample},
    traits::AudioNode,
};

/// Mutable output buffers for a single node's `process` call.
///
/// Wraps a fixed-size array of `RefMut` guards (one per output slot, indexed
/// by `PortId`).  `get_mut` returns a plain `&mut [Sample]` so node
/// implementations never need to interact with `RefMut` directly.
///
/// ## Sequential access
///
/// `get_mut` borrows `&mut self`, so only one output buffer can be held at a
/// time.  For nodes with a single output (the common case) this is invisible.
/// For nodes with multiple outputs, write to each output inside its own block:
///
/// ```ignore
/// if let Some(left) = outputs.get_mut(LEFT_PORT) { /* write */ }
/// if let Some(right) = outputs.get_mut(RIGHT_PORT) { /* write */ }
/// ```
pub struct OutputBuffers<'pool> {
    guards: [Option<RefMut<'pool, [Sample]>>; PortId::MAX_PORT_ID],
}

impl<'pool> OutputBuffers<'pool> {
    /// Returns a mutable view of the output buffer for `port_id`, or `None`
    /// if the port is not connected.
    pub fn get_mut(&mut self, port_id: impl Into<PortId>) -> Option<&mut [Sample]> {
        self.guards.get_mut(**port_id.into())?.as_deref_mut()
    }
}

/// Per-node bridge between the graph's `NodePortMap` + `BufferPool` and a
/// node's `process` call.  Created fresh each frame in `run()`; never stored.
///
/// The key entry point is [`call_process`], which acquires all `RefCell`
/// borrows, builds the plain-slice arrays, and dispatches to the node.
///
/// ## Self-loop safety
///
/// If an output port is connected back to the same node's input port, both
/// `borrow_mut()` and `borrow()` will target the same `RefCell`.  `RefCell`
/// panics rather than producing undefined behaviour.
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
    /// plain slices, and calls `node.process(inputs, outputs)`.
    ///
    /// All borrows are held for the duration of the `process` call and
    /// released when this method returns.
    ///
    /// ## Stack allocation
    ///
    /// Three fixed-size arrays of `PortId::MAX_PORT_ID` elements are placed
    /// on the stack (~12 KB at the default limit of 256).  Unconnected ports
    /// are `None` and hold no borrow.
    pub fn call_process(
        &self,
        node: &mut dyn AudioNode,
    ) -> Result<(), crate::errors::AudioNodeRunError> {
        // Acquire a shared borrow for every connected input port.
        let input_guards: [Option<Ref<'pool, [Sample]>>; PortId::MAX_PORT_ID] =
            core::array::from_fn(|i| {
                self.inputs
                    .get(i)?
                    .as_ref()
                    .and_then(|id| self.pool.get(id))
                    .map(|c| Ref::map(c.borrow(), |b| b.as_ref()))
            });

        // Deref each guard to a plain shared slice.
        let inputs: [Option<&[Sample]>; PortId::MAX_PORT_ID] =
            core::array::from_fn(|i| input_guards[i].as_deref());

        // Acquire an exclusive borrow for every connected output port.
        let mut outputs = OutputBuffers {
            guards: core::array::from_fn(|i| {
                self.outputs
                    .get(i)?
                    .as_ref()
                    .and_then(|id| self.pool.get(id))
                    .map(|c| RefMut::map(c.borrow_mut(), |b| b.as_mut()))
            }),
        };

        node.process(&inputs, &mut outputs, self.block_size)
    }
}
