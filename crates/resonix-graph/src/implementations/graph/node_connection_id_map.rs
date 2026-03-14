use alloc::boxed::Box;

use crate::primitives::{ConnectionId, ExternalConnectionId};

/// Per-node storage of which `ConnectionId` backs each port slot.
/// Allocated once at `add()` time; updated during `connect()`.
/// A `None` entry means the port is unconnected or external.
pub(in crate::implementations::graph) struct NodeConnectionIdMap {
    pub input_port_slots: Box<[Option<ConnectionId>]>,
    pub output_port_slots: Box<[Option<ConnectionId>]>,
    /// Indexed by PortId. `Some(ext_id)` means the slot is an external input port.
    /// Same length as `input_port_slots`. Copied directly into `CompiledStep` at compile time.
    pub external_input_slots: Box<[Option<ExternalConnectionId>]>,
    /// Indexed by PortId. `Some(ext_id)` means the slot is an external output port.
    /// Same length as `output_port_slots`. Copied directly into `CompiledStep` at compile time.
    pub external_output_slots: Box<[Option<ExternalConnectionId>]>,
}
