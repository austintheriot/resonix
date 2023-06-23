use async_trait::async_trait;
use petgraph::prelude::NodeIndex;

use crate::{AddConnectionError, Node};

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Node could not be found in the audio graph for index {node_index:?}. Are you sure you added it?")]
    NodeNotFound { node_index: NodeIndex },
    #[error("Node connection from {parent_node_name:?} to {child_node_name:?} failed. Expected `from_index` to be a max of {expected_from_index:?} and `to_index`  to be a max of {expected_to_index:?}. Received `from_index`  of {from_index:?} and `to_index` of {to_index:?}")]
    IncorrectIndex {
        expected_from_index: usize,
        expected_to_index: usize,
        from_index: usize,
        to_index: usize,
        parent_node_name: String,
        child_node_name: String,
    },
    #[error("Node connection failed. Original error: {0:?}")]
    AddConnectionError(#[from] AddConnectionError),
}

#[async_trait]
pub trait ProcessorInterface {
    async fn add_node<N: Node + 'static>(&mut self, node: N) -> Result<NodeIndex, N>;

    async fn connect(&mut self, node_1: NodeIndex, node_2: NodeIndex) -> Result<&mut Self, ConnectError>;
}
