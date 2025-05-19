use crate::Connectable;

pub trait ResonixGraph {
    fn add(&mut self, connectable: Connectable);
}

#[cfg(test)]
mod tests {
    use core::clone;

    use alloc::{borrow::ToOwned, vec::Vec};

    use crate::{ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult};

    #[test]
    fn implementation_ideas() {
        struct ConstantNode;

        impl ResonixAudioNode for ConstantNode {
            fn next(&mut self) -> ResonixDataResult {
                let mut connection_data: Vec<ResonixDataList> = Vec::with_capacity(1);
                let mut values = Vec::with_capacity(1);
                values.push(ResonixData::F32(1.0));
                connection_data.push(ResonixDataList::from([ResonixData::F32(1.0)]));
                connection_data.into()
            }

            fn assign_inputs(&mut self, _inputs: crate::ResonixDataResult) {
                // it takes no inputs
                unimplemented!()
            }
        }

        struct MultiplyNode {
            inputs: ResonixDataResult,
        }

        impl ResonixAudioNode for MultiplyNode {
            fn next(&mut self) -> ResonixDataResult {
                let new_data = self.inputs.to_owned();
                let resonix_data_list_vec: Vec<ResonixDataList> = new_data
                    .into_inner()
                    .into_iter()
                    .map(|connection_data| {
                        let resonix_data_vec: Vec<ResonixData> = connection_data
                            .into_inner()
                            .into_iter()
                            .map(|data| match data {
                                ResonixData::F32(value) => ResonixData::F32(value * 2.0),
                            })
                            .collect();
                        let new_resonix_data_list: ResonixDataList = resonix_data_vec.into();
                        new_resonix_data_list
                    })
                    .collect();
                resonix_data_list_vec.into()
            }

            fn assign_inputs(&mut self, inputs: crate::ResonixDataResult) {
                self.inputs = inputs;
            }
        }

        let mut constant_node = ConstantNode;

        let connection_data = constant_node.next();
        assert_eq!(
            *connection_data.get(0).unwrap().get(0).unwrap(),
            ResonixData::F32(1.0)
        )

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
