use crate::ResonixId;

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

    pub fn node_id(&self) -> ResonixId {
        self.node_id
    }
}
