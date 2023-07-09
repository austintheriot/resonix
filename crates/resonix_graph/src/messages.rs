use petgraph::{prelude::NodeIndex, stable_graph::EdgeIndex};
use resonix_core::NumChannels;

use crate::{AddConnectionError, Node, NodeUid};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum MessageError {
    #[error("A message was sent to the `Processor` in the audio thread to connect 2 nodes, but no corresponding response was received")]
    NoMatchingMessageReceived,
    #[error("A response was received from the `Processor` on the main thread, but it was not the expected response")]
    WrongResponseReceived,
    #[error("Error occured while connecting nodes: {0}")]
    ConnectError(#[from] ConnectError),
    #[error("Error occured while adding node: {0}")]
    AddNodeError(#[from] AddNodeError),
    #[error("Error occured while updating node: {0}")]
    UpdateNodeError(#[from] UpdateNodeError),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ConnectError {
    #[error("Node could not be found in the audio graph for index {node_index:?}. Are you sure you added it?")]
    NodeNotFound { node_index: NodeIndex },
    #[error("Node's UID could not be found  {node_uid:?}. Are you sure you added it?")]
    NodeUidNotFound { node_uid: NodeUid },
    #[error("Node connection from {parent_node_name:?} to {child_node_name:?} failed. Expected `from_index` to be a max of {expected_from_index:?} and `to_index`  to be a max of {expected_to_index:?}. Received `from_index`  of {from_index:?} and `to_index` of {to_index:?}")]
    IncorrectIndex {
        expected_from_index: usize,
        expected_to_index: usize,
        from_index: usize,
        to_index: usize,
        parent_node_name: String,
        child_node_name: String,
    },
    #[error("Node connection from parent node {parent_node_name:?} to child node {child_node_name:?} failed. Parent node has {parent_node_num_outgoing_channels:?} outgoing channels while child node has {child_node_num_incoming_channels:?} incoming channels")]
    IncompatibleNumChannels {
        parent_node_num_outgoing_channels: NumChannels,
        child_node_num_incoming_channels: NumChannels,
        parent_node_name: String,
        child_node_name: String,
    },
    #[error("Node connection failed. Original error: {0:?}")]
    AddConnectionError(#[from] AddConnectionError),
    #[error("A message was sent to the `Processor` in the audio thread to connect 2 nodes, but no corresponding response was received")]
    NoMatchingMessageReceived,
    #[error("Can't create a connection between {parent_node_name:?} and {child_node_name:?}, because doing so creates a cyclical graph")]
    GraphCycleFound {
        parent_node_name: String,
        child_node_name: String,
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AddNodeError {
    #[error("Cannot add {name:?} to the audio graph, since it has already been added.")]
    AlreadyExists { name: String },
    #[error("A message was sent to the `Processor` in the audio thread to add a node, but no corresponding response was received")]
    NoMatchingMessageReceived,
}

/// When no DAC has been initialized yet, the audio graph can be run on the main thread,
/// but once the DAC is initialized, the DAC receive ownership of the
/// audio graph to run it in the high-priority audio thread.
///
/// Once the Processor (audio graph) has been sent to the audio thread,
/// all edits to the audio graph have to be done
/// via message between the audio thread and main thread.
#[derive(Debug, PartialEq)]
pub(crate) enum ProcessorMessageRequest<N: Node + 'static> {
    AddNode {
        request_id: u32,
        node: N,
    },
    Connect {
        request_id: u32,
        parent_node_uid: NodeUid,
        child_node_uid: NodeUid,
    },
    UpdateNode {
        request_id: u32,
        request: NodeMessageRequest,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum ProcessorMessageResponse {
    AddNode {
        request_id: u32,
        result: Result<NodeUid, AddNodeError>,
    },
    Connect {
        request_id: u32,
        result: Result<EdgeIndex, ConnectError>,
    },
    UpdateNode {
        request_id: u32,
        result: Result<(), UpdateNodeError>,
    },
}

impl ProcessorMessageResponse {
    pub fn request_id(&self) -> u32 {
        match self {
            ProcessorMessageResponse::AddNode { request_id, .. } => *request_id,
            ProcessorMessageResponse::Connect { request_id, .. } => *request_id,
            ProcessorMessageResponse::UpdateNode { request_id, .. } => *request_id,
        }
    }
}

/// These are internal errors that can occur while sending messages back
/// and forth between a node on the main thread and the audio thread
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum UpdateNodeError {
    #[error("No corresponding node found in the graph for node with uuid {uid:}")]
    NodeNotFound { uid: NodeUid },
    #[error("Node message was sent for the wrong node type")]
    WrongNodeType { uid: NodeUid },
}

/// These messages are sent by individual `NodeHandle` instances
/// to effect some change in the audio graph.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum NodeMessageRequest {
    SineSetFrequency { node_uid: u32, new_frequency: f32 },
}
