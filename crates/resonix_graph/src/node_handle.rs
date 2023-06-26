use std::marker::PhantomData;

use async_channel::{Sender, Receiver};
use uuid::Uuid;
use petgraph::prelude::NodeIndex;

use crate::{messages::{NodeMessageRequest, NodeMessageResponse}, Node};

#[derive(thiserror::Error, Debug)]
pub enum NodeHandleMessageError {
    #[error("Unexpected response received in node_handle while trying to communicate with Processor")]
    UnexpectedResponseReceived
}

/// This struct can be cheaply cloned
#[derive(Debug, Clone)]
pub struct NodeHandle<NodeType: Node> {
    pub(crate) uuid: Uuid,
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
