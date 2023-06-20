use crate::Node;

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Node connection from {parent_node_name:?} to {child_node_name:?} failed. Expected `from_index` to be a max of {expected_from_index:?} and `to_index`  to be a max of {expected_to_index:?}. Received `from_index`  of {from_index:?} and `to_index` of {to_index:?}")]
    IncorrectIndex {
        expected_from_index: usize,
        expected_to_index: usize,
        from_index: usize,
        to_index: usize,
        parent_node_name: String,
        child_node_name: String,
    },
}

/// Allows two nodes to be joined to each other with a connection in
/// the audio graph.
///
/// The node that is calling `connect` is always the parent node,
/// and the second, `other_node` passed as an argument is always
/// the child node that is receiving a new connection.
pub trait Connect
where
    Self: Clone + Node,
{
    /// connects the current Node's output to the input of the next node
    fn connect<N: Node + Connect>(&self, other_node: &N) -> Result<&Self, ConnectError> {
        self.connect_nodes_with_indexes(Default::default(), other_node, Default::default())
    }

    fn connect_nodes_with_indexes<N: Node + Connect + Clone>(
        &self,
        from_index: usize,
        other_node: &N,
        to_index: usize,
    ) -> Result<&Self, ConnectError>;

    fn check_index_out_of_bounds<N: Node + Connect + Clone>(
        &self,
        from_index: usize,
        other_node: &N,
        to_index: usize,
    ) -> Result<(), ConnectError> {
        if self.connection_index_out_of_bounds(from_index, other_node, to_index) {
            return Err(ConnectError::IncorrectIndex {
                expected_from_index: self.num_inputs() - 1,
                expected_to_index: other_node.num_inputs() - 1,
                from_index,
                to_index,
                parent_node_name: self.name(),
                child_node_name: other_node.name(),
            });
        }

        Ok(())
    }

    fn connection_index_out_of_bounds<N: Node + Connect + Clone>(
        &self,
        from_index: usize,
        other_node: &N,
        to_index: usize,
    ) -> bool {
        from_index >= self.num_outputs() || to_index >= other_node.num_inputs()
    }
}
