use alloc::boxed::Box;

use crate::primitives::ConnectionId;

/// Per-node storage of which `ConnectionId` backs each port slot.
/// Allocated once at `add()` time; updated during `connect()`.
/// A `None` entry means the port is unconnected.
pub(in crate::implementations::graph) struct NodeConnectionIdMap {
    pub input_port_slots: Box<[Option<ConnectionId>]>,
    pub output_port_slots: Box<[Option<ConnectionId>]>,
}
