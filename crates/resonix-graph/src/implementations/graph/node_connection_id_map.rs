use alloc::boxed::Box;

use crate::primitives::{ConnectionId, ExternalConnectionId};

/// Per-node storage of which `ConnectionId` backs each port slot.
/// Allocated once at `add()` time; updated during `connect()`.
/// A `None` entry means the port is unconnected (or is an external port for `input/output_port_slots`).
pub(in crate::implementations::graph) struct NodeConnectionIdMap {
    /// Indexed by PortId. Stores the `ConnectionId` for each internal input port;
    /// external input port slots are `None` here (tracked in `external_input_slots`).
    pub internal_input_port_slots: Box<[Option<ConnectionId>]>,
    /// Indexed by PortId. Stores the `ConnectionId` for each internal output port;
    /// external output port slots are `None` here (tracked in `external_output_slots`).
    pub internal_output_port_slots: Box<[Option<ConnectionId>]>,
    /// Indexed by PortId. `Some(ext_id)` at each external input port position.
    pub external_input_slots: Box<[Option<ExternalConnectionId>]>,
    /// Indexed by PortId. `Some(ext_id)` at each external output port position.
    pub external_output_slots: Box<[Option<ExternalConnectionId>]>,
}
