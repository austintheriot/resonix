use petgraph::{prelude::NodeIndex, stable_graph::EdgeIndex};

use crate::{AddNodeError, ConnectError, Node};

/// When no DAC has been initialized yet, the audio graph can be run on the main thread,
/// but once the DAC is initialized, the DAC receive ownership of the
/// audio graph to run it in the high-priority audio thread.
///
/// Once the Processor (audio graph) has been sent to the audio thread,
/// all edits to the audio graph have to be done
/// via message between the audio thread and main thread.
#[derive(Debug)]
pub(crate) enum MessageRequest<N: Node + 'static> {
    AddNode {
        id: u32,
        node: N,
    },
    Connect {
        id: u32,
        parent_node_index: NodeIndex,
        child_node_index: NodeIndex,
    },
}

#[derive(Debug)]
pub(crate) enum MessageResponse {
    AddNode {
        id: u32,
        result: Result<NodeIndex, AddNodeError>,
    },
    Connect {
        id: u32,
        result: Result<EdgeIndex, ConnectError>,
    },
}
