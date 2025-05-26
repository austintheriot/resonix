use alloc::boxed::Box;

use crate::primitives::Id;
use crate::traits::Audio;
use crate::traits::GetPriority;
use crate::traits::{AudioNode, GetNodeId, Param, ParamNode};

/// Wrapper type around the `AudioNode` and `ParamNode` types
/// for easier, opaque handling in the Graph.
///
/// Does require dynamic dispatch, which is a downside, but dealing with
/// these dynamic types otherwise would be difficult.
pub enum Node {
    AudioNode(Box<dyn AudioNode>),
    ParamNode(Box<dyn ParamNode>),
}

impl GetNodeId for Node {
    fn node_id(&self) -> Id {
        match self {
            Node::AudioNode(audio_node) => audio_node.node_id(),
            Node::ParamNode(param_node) => param_node.node_id(),
        }
    }
}

impl GetPriority for Node {
    fn get_priority(&self) -> super::Priority {
        match self {
            Node::AudioNode(audio_node) => audio_node.get_priority(),
            Node::ParamNode(param_node) => param_node.get_priority(),
        }
    }
}

impl<A> From<Audio<A>> for Node
where
    A: AudioNode + 'static,
{
    fn from(value: Audio<A>) -> Self {
        Node::AudioNode(Box::new(value.into_inner()))
    }
}

impl<P> From<Param<P>> for Node
where
    P: ParamNode + 'static,
{
    fn from(value: Param<P>) -> Self {
        Node::ParamNode(Box::new(value.into_inner()))
    }
}
