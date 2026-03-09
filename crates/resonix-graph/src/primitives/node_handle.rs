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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::IntMap;
    use nohash_hasher::BuildNoHashHasher;

    fn empty_connection_map() -> IntMap<PortId, ConnectionId> {
        IntMap::with_hasher(BuildNoHashHasher::default())
    }

    #[test]
    fn node_id_trait_returns_the_inner_id_value() {
        let node_id = NodeId::new(42);
        let handle = NodeHandle::new(node_id, (), empty_connection_map(), empty_connection_map());
        assert_eq!(handle.node_id(), *node_id);
    }

    #[test]
    fn deref_provides_access_to_port_descriptors() {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct TestPortDescriptors;

        let node_id = NodeId::new(1);
        let port_descriptors = TestPortDescriptors;
        let handle = NodeHandle::new(
            node_id,
            port_descriptors,
            empty_connection_map(),
            empty_connection_map(),
        );
        assert_eq!(*handle, port_descriptors);
    }

    #[test]
    fn external_input_connection_ids_returns_the_map_provided_at_construction() {
        let node_id = NodeId::new(0);
        let mut input_connections = empty_connection_map();
        input_connections.insert(PortId::new(0), ConnectionId::new(10));

        let handle = NodeHandle::new(node_id, (), input_connections, empty_connection_map());

        assert_eq!(
            handle.external_input_connection_ids().get(&PortId::new(0)),
            Some(&ConnectionId::new(10))
        );
    }

    #[test]
    fn external_output_connection_ids_returns_the_map_provided_at_construction() {
        let node_id = NodeId::new(0);
        let mut output_connections = empty_connection_map();
        output_connections.insert(PortId::new(0), ConnectionId::new(20));

        let handle = NodeHandle::new(node_id, (), empty_connection_map(), output_connections);

        assert_eq!(
            handle.external_output_connection_ids().get(&PortId::new(0)),
            Some(&ConnectionId::new(20))
        );
    }
}
