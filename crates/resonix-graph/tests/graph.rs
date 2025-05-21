mod common;

mod tests {

    use crate::common::nodes::{ConstantNode, MultiplyNode};
    use resonix_graph::{GenerateId, ResonixAudioNode, ResonixData, ResonixDataResult};

    #[test]
    fn graph_can_accept_nodes() {}

    #[test]
    fn nodes_can_receive_and_generate_values() {
        #[derive(Default)]
        struct IdGenerator(usize);

        impl GenerateId for IdGenerator {
            fn generate_id(&mut self) -> resonix_graph::ResonixId {
                let current_node_id = self.0;
                self.0 += 1;
                current_node_id.into()
            }
        }

        let mut id_generator = IdGenerator::default();

        let mut constant_node = ConstantNode::new(&mut id_generator);
        let connection_data = constant_node.next();
        assert_eq!(
            *connection_data.first().unwrap().first().unwrap(),
            ResonixData::F32(1.0)
        );

        let mut multiply_node = MultiplyNode::new(&mut id_generator, 2.0);
        multiply_node.assign_inputs(ResonixDataResult::from_value(3));
        let connection_data = multiply_node.next();
        assert_eq!(
            *connection_data.first().unwrap().first().unwrap(),
            ResonixData::I32(6)
        );

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
