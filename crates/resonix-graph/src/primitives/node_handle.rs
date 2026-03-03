use core::ops::Deref;

use crate::traits::GetNodeId;
use crate::{primitives::Id, utils::IntMap};

use super::{ConnectionId, NodeId, PortId};

/// Indicates the presence of a Node in the Graph.
/// Derefs to the Node's PortDescriptors to allow making
/// connections easier after Nodes have already been added to the Graph.
#[derive(Debug, PartialEq)]
pub struct NodeHandle<PortDescriptor> {
    pub(crate) node_id: NodeId,
    pub(crate) port_descriptors: PortDescriptor,
    pub(crate) external_input_connection_ids: IntMap<PortId, ConnectionId>,
    pub(crate) external_output_connection_ids: IntMap<PortId, ConnectionId>,
}

impl<PortDescriptors> NodeHandle<PortDescriptors> {
    pub fn new(
        node_id: NodeId,
        port_descriptors: PortDescriptors,
        external_input_connection_ids: IntMap<PortId, ConnectionId>,
        external_output_connection_ids: IntMap<PortId, ConnectionId>,
    ) -> Self {
        Self {
            node_id,
            port_descriptors,
            external_input_connection_ids,
            external_output_connection_ids,
        }
    }

    pub fn external_input_connection_ids(&self) -> &IntMap<PortId, ConnectionId> {
        &self.external_input_connection_ids
    }

    pub fn external_output_connection_ids(&self) -> &IntMap<PortId, ConnectionId> {
        &self.external_output_connection_ids
    }
}

impl<PortDescriptors> GetNodeId for NodeHandle<PortDescriptors> {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

impl<PortDescriptors> Deref for NodeHandle<PortDescriptors> {
    type Target = PortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}
