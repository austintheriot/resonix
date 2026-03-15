use alloc::boxed::Box;

use crate::primitives::{ConnectionId, ExternalConnectionId};

/// Per-node storage of which `ConnectionId` backs each port slot.
/// Allocated once at `add()` time; updated during `connect()`.
/// A `None` entry means the port is unconnected.
pub(in crate::implementations::graph) struct NodeConnectionIdMap {
    pub internal_input_port_slots: Box<[Option<ConnectionId>]>,
    pub internal_output_port_slots: Box<[Option<ConnectionId>]>,
    /// Indexed by external PortId (0..n). One entry per external input port.
    /// Copied directly into `CompiledStep` at compile time.
    pub external_input_slots: Box<[ExternalConnectionId]>,
    /// Indexed by external PortId (0..n). One entry per external output port.
    /// Copied directly into `CompiledStep` at compile time.
    pub external_output_slots: Box<[ExternalConnectionId]>,
}
