use std::marker::PhantomData;

use async_channel::{Receiver, Sender};
use petgraph::prelude::NodeIndex;
use uuid::Uuid;

use crate::{
    messages::{NodeMessageError, NodeMessageRequest, NodeMessageResponse},
    Node,
};

#[derive(thiserror::Error, Debug)]
pub enum NodeHandleMessageError {
    #[error("A message was sent from the `NodeHandle` to the processor, but no corresponding message was received")]
    NoMatchingMessageReceived,
    #[error("Error occurred while communicating with Processor. Original error: {0:?}")]
    NodeMessageError(#[from] NodeMessageError),
}

/// The `NodeHandle` allows mutating audio a node's data from
/// the main thread, even after that node has been sent to
/// the audio thread. `NodeHandle` implements specific functionality
/// for whatever generic `node_type` the `NodeHandle` is.
///
/// This is accomplished by sending messages between the main
/// thread and the audio thread.
///
/// All audio graph mutations are processed in the order in which
/// they were received.
///
/// This struct can be safely and cheaply cloned
#[derive(Debug, Clone)]
pub struct NodeHandle<NodeType: Node> {
    pub(crate) uid: u32,
    pub(crate) node_index: NodeIndex,
    pub(crate) node_request_tx: Sender<NodeMessageRequest>,
    pub(crate) node_response_rx: Receiver<NodeMessageResponse>,
    pub(crate) node_type: PhantomData<NodeType>,
}

impl<N: Node> AsRef<NodeIndex> for NodeHandle<N> {
    fn as_ref(&self) -> &NodeIndex {
        &self.node_index
    }
}
