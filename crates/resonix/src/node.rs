use std::fmt::Debug;

use crate::NodeType;

pub trait Node
    where Self: Debug,
 {
    fn process(&mut self, inputs: &[f32], outputs: &mut [f32]);

    fn node_type(&self) -> NodeType;

    fn num_inputs(&self) -> usize;

    fn num_outputs(&self) -> usize;
}

pub type BoxedNode = Box<dyn Node>;

#[cfg(test)]
mod test_node {
    use petgraph::{Graph, visit::Dfs, stable_graph::NodeIndex};

    use crate::{BoxedNode, Sine, PassThrough, Record};

    #[test]
    pub fn it_should_allow_processing_audio() {
        type G = Graph<BoxedNode, ()>;
        let mut graph: Graph<BoxedNode, ()> = Graph::new();
        let sine = Sine::new_with_config(44100, 440.0);
        let origin = graph.add_node(Box::new(sine));
        let pass_through = graph.add_node(Box::new(PassThrough));
        let record = graph.add_node(Box::new(Record::new()));
        graph.add_edge(origin, pass_through, ());
        graph.add_edge(pass_through, record, ());

        fn process(node_index: NodeIndex, graph: &mut G) {
            let mut node = graph[node_index];
            let mut input_sum = 0.0;
            for input_index in graph.neighbors_directed(node_index, Direction::Incoming) {
                input_sum= 
            }
        }

        let mut dfs = Dfs::new(&graph, origin);
        while let Some(node_index) = dfs.next(&graph) {
            let mut node = graph[node_index];
            let node_type = node.node_type();
            let neighbors = graph.neighbors_directed(todo!(), Direction);
        }
    }
}