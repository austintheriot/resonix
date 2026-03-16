use alloc::boxed::Box;

use crate::primitives::{ConnectionId, ExternalConnectionId};

/// Per-node storage of which `ConnectionId` backs each port slot.
/// Allocated once at `add()` time; updated during `connect()`.
/// A `None` entry means the port is unconnected (or is an external port for `input/output_port_slots`).
pub(in crate::implementations::graph) struct NodeConnectionIdMap {
    /// Indexed by PortId (combined input namespace). Stores the `ConnectionId` for each
    /// internal input port; external input port slots are `None` here (tracked in
    /// `external_input_slots`).
    pub input_port_slots: Box<[Option<ConnectionId>]>,
    /// Indexed by PortId (combined output namespace). Stores the `ConnectionId` for each
    /// internal output port; external output port slots are `None` here (tracked in
    /// `external_output_slots`).
    pub output_port_slots: Box<[Option<ConnectionId>]>,
    /// Indexed by PortId (combined input namespace). `Some(ext_id)` at each external input
    /// port position, `None` at internal input port positions.
    pub external_input_slots: Box<[Option<ExternalConnectionId>]>,
    /// Indexed by PortId (combined output namespace). `Some(ext_id)` at each external output
    /// port position, `None` at internal output port positions.
    pub external_output_slots: Box<[Option<ExternalConnectionId>]>,
}
