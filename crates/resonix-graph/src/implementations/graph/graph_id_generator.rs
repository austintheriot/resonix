use crate::{primitives::Id, traits::GenerateId};

#[derive(Default)]
pub(in crate::implementations::graph) struct GraphIdGenerator {
    current_node_id: usize,
}

impl GenerateId for GraphIdGenerator {
    fn generate_id(&mut self) -> Id {
        let current_node_id = self.current_node_id;
        self.current_node_id += 1;
        current_node_id.into()
    }
}
