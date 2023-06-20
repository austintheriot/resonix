use crate::Node;

pub trait Connect
where
    Self: Clone,
{
    /// connects the current Node's output to the input of the next node
    fn connect<N: Node + Connect>(&self, other_node: &N) -> &Self {
        self.connect_nodes_with_indexes(Default::default(), other_node, Default::default())
    }

    fn connect_nodes_with_indexes<N: Node + Connect + Clone>(
        &self,
        from_index: usize,
        other_node: &N,
        to_index: usize,
    ) -> &Self;
}
