use core::ops::Deref;

use crate::{GetNodeId, ResonixId};

/// Indicates the presence of a Node in the Graph
/// Derefs to the Node's PortDescriptors to allow making
/// connections easier after Nodes have already been added to the Graph.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixNodeHandle<PortDescriptor> {
    node_id: ResonixId,
    port_descriptors: PortDescriptor,
}

impl<PortDescriptors> ResonixNodeHandle<PortDescriptors> {
    pub fn new(node_id: ResonixId, port_descriptors: PortDescriptors) -> Self {
        Self {
            node_id,
            port_descriptors,
        }
    }
}

impl<PortDescriptors> GetNodeId for ResonixNodeHandle<PortDescriptors> {
    fn node_id(&self) -> ResonixId {
        self.node_id
    }
}

impl<PortDescriptors> Deref for ResonixNodeHandle<PortDescriptors> {
    type Target = PortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}
