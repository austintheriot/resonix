use crate::{
    primitives::{ExternalConnectionId, Id},
    traits::GenerateId,
};

#[derive(Default)]
pub(in crate::implementations::graph) struct GraphIdGenerator {
    current_node_id: usize,
    current_external_id: usize,
}

impl GraphIdGenerator {
    pub fn generate_external_id(&mut self) -> ExternalConnectionId {
        let id = self.current_external_id;
        self.current_external_id += 1;
        ExternalConnectionId::new(id)
    }
}

impl GenerateId for GraphIdGenerator {
    fn generate_id(&mut self) -> Id {
        let current_node_id = self.current_node_id;
        self.current_node_id += 1;
        current_node_id.into()
    }
}
