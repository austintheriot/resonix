use petgraph::{prelude::NodeIndex, stable_graph::EdgeIndex};
use uuid::Uuid;

use crate::{AddNodeError, ConnectError, Node};

/// When no DAC has been initialized yet, the audio graph can be run on the main thread,
/// but once the DAC is initialized, the DAC receive ownership of the
/// audio graph to run it in the high-priority audio thread.
///
/// Once the Processor (audio graph) has been sent to the audio thread,
/// all edits to the audio graph have to be done
/// via message between the audio thread and main thread.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProcessorMessageRequest<N: Node + 'static> {
    AddNode {
        request_id: u32,
        node: N,
    },
    Connect {
        request_id: u32,
        parent_node_index: NodeIndex,
        child_node_index: NodeIndex,
    },
}

#[derive(Debug)]
pub(crate) enum ProcessorMessageResponse {
    AddNode {
        request_id: u32,
        result: Result<NodeIndex, AddNodeError>,
    },
    Connect {
        request_id: u32,
        result: Result<EdgeIndex, ConnectError>,
    },
}

/// These are internal errors that can occur while sending messages back
/// and forth between a node on the main thread and the audio thread
#[derive(thiserror::Error, Debug)]
pub enum NodeMessageError {
    #[error("No corresponding node found in the graph for node with node_uid {node_uid:?} at node index {node_index:?}")]
    NodeNotFound { node_uid: u32, node_index: NodeIndex },
    #[error("Node message was sent for the wrong node type")]
    WrongNodeType { node_uid: u32, node_index: NodeIndex },
}

/// These messages are sent by individual `NodeHandle` instances
/// to effect some change in the audio graph.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum NodeMessageRequest {
    SineSetFrequency {
        node_uid: u32,
        node_index: NodeIndex,
        new_frequency: f32,
    },
}

/// These messages acknowledge whether a change to an audio graph node was successful or not
#[derive(Debug)]
pub(crate) enum NodeMessageResponse {
    SineSetFrequency {
        node_uid: u32,
        result: Result<(), NodeMessageError>,
    },
}
