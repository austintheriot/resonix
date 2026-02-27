use core::cell::{Ref, RefMut};

use crate::primitives::{BlockSize, BufferPool, ConnectionId, PortId, Sample};

/// Short-lived per-node context created during each `run()` call.
///
/// Holds borrowed references into the graph's buffer pool and the pre-computed
/// connection-ID index for this node's ports.  Constructed once per node per
/// `run()` invocation; never stored across frames.
///
/// ## Self-loop safety
///
/// If a node's output port is connected back to one of its own input ports,
/// both `output()` and `input()` will attempt to borrow the *same* `RefCell`.
/// `RefCell` will **panic** rather than produce undefined behaviour, making
/// the bug immediately visible.
///
/// ## Node removal safety
///
/// This type holds a `&'pool BufferPool` reference rather than raw pointers,
/// so it is impossible for a buffer to be freed while a context is live.
pub struct NodeProcessContext<'pool> {
    /// Connection IDs for each input slot, indexed by port_id.
    /// `None` means the port is not connected.
    inputs: &'pool [Option<ConnectionId>],
    /// Connection IDs for each output slot, indexed by port_id.
    /// `None` means the port is not connected.
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

    /// Returns a shared view of the input buffer for `port_id`, or `None` if
    /// the port is not connected.
    ///
    /// The returned `Ref` keeps the `RefCell` borrowed for its lifetime.
    /// Calling `output()` with a port that shares a buffer with this input
    /// (a self-loop) will panic.
    pub fn input(&self, port_id: impl Into<PortId>) -> Option<Ref<'pool, [Sample]>> {
        let conn_id = self.inputs.get(**port_id.into())?.as_ref()?;
        let cell = self.pool.get(conn_id)?;
        Some(Ref::map(cell.borrow(), |b| b.as_ref()))
    }

    /// Returns a mutable view of the output buffer for `port_id`, or `None`
    /// if the port is not connected.
    ///
    /// The returned `RefMut` keeps the `RefCell` mutably borrowed for its
    /// lifetime.  Calling `input()` or `output()` with a port that shares
    /// the same buffer (self-loop or double-output call) will panic.
    pub fn output(&self, port_id: impl Into<PortId>) -> Option<RefMut<'pool, [Sample]>> {
        let conn_id = self.outputs.get(**port_id.into())?.as_ref()?;
        let cell = self.pool.get(conn_id)?;
        Some(RefMut::map(cell.borrow_mut(), |b| b.as_mut()))
    }

    pub fn block_size(&self) -> BlockSize {
        self.block_size
    }
}
