use crate::Connectable;

pub trait ResonixGraph {
    fn add(&mut self, connectable: Connectable);
}

#[cfg(test)]
mod tests {
    #[test]
    fn implementation_ideas() {
        /*
        // SIMPLEST OPTION, modelled after Web Audio API
        let graph = Graph::new();
        let node_a = NodeA::new();
        let node_b = NodeB::new();
        node_a.connect(node_b);
        graph.add(node_a);

        let graph = NodeA::new().connect(NodeB::new());

        // USING WITH-SYNTAX-compose sub-grapsh
        let mut graph = Graph::new();
        graph.add_with(|| {
            NodeA::new()
                .connect(NodeB::new())
                .connect(
                    NodeC::new().connect(NodeD::new())
                )
        })

        // USING HASH_MAP-like Result Data-types
        graph
            .add_node(SomeNode::new())
            .connect_to(SomeOtherNode::new())

         */
    }
}
