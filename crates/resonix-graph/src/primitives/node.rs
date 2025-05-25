use alloc::boxed::Box;

use crate::{Audio, GetNodeId, GetPriority, Param, ResonixAudioNode, ResonixId, ResonixParamNode};

/// Wrapper type around the `ResonixAudioNode` and `ResonixParamNode` types
/// for easier, opaque handling in the Graph.
///
/// Does require dynamic dispatch, which is a downside, but dealing with
/// these dynamic types otherwise would be difficult.
pub enum Node {
    AudioNode(Box<dyn ResonixAudioNode>),
    ParamNode(Box<dyn ResonixParamNode>),
}

impl GetNodeId for Node {
    fn node_id(&self) -> ResonixId {
        match self {
            Node::AudioNode(resonix_audio_node) => resonix_audio_node.node_id(),
            Node::ParamNode(resonix_param_node) => resonix_param_node.node_id(),
        }
    }
}

impl GetPriority for Node {
    fn get_priority(&self) -> super::Priority {
        match self {
            Node::AudioNode(resonix_audio_node) => resonix_audio_node.get_priority(),
            Node::ParamNode(resonix_param_node) => resonix_param_node.get_priority(),
        }
    }
}

impl<A> From<Audio<A>> for Node
where
    A: ResonixAudioNode + 'static,
{
    fn from(value: Audio<A>) -> Self {
        Node::AudioNode(Box::new(value.into_inner()))
    }
}

impl<P> From<Param<P>> for Node
where
    P: ResonixParamNode + 'static,
{
    fn from(value: Param<P>) -> Self {
        Node::ParamNode(Box::new(value.into_inner()))
    }
}
