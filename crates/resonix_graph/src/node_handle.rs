use std::marker::PhantomData;

use async_channel::{Receiver, Sender};
use petgraph::prelude::NodeIndex;

use crate::{messages::MessageError, Node, NodeUid};

#[derive(thiserror::Error, Debug)]
pub enum NodeHandleMessageError {
    #[error("A message was sent from the `NodeHandle` to the processor, but no corresponding message was received")]
    NoMatchingMessageReceived,
    #[error("Error occurred while communicating with Processor. Original error: {0:?}")]
    MessageError(#[from] MessageError),
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
    pub(crate) uid: NodeUid,
    pub(crate) node_type: PhantomData<NodeType>,
}

impl<N: Node> AsRef<NodeUid> for NodeHandle<N> {
    fn as_ref(&self) -> &NodeUid {
        &self.uid
    }
}