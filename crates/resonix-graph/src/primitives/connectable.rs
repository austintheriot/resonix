use alloc::boxed::Box;

use crate::{Audio, GetNodeId, Param, ResonixAudioNode, ResonixId, ResonixParamNode};

pub enum Connectable {
    AudioNode(Box<dyn ResonixAudioNode>),
    ParamNode(Box<dyn ResonixParamNode>),
}

impl GetNodeId for Connectable {
    fn node_id(&self) -> ResonixId {
        match self {
            Connectable::AudioNode(resonix_audio_node) => resonix_audio_node.node_id(),
            Connectable::ParamNode(resonix_param_node) => resonix_param_node.node_id(),
        }
    }
}

impl<A> From<Audio<A>> for Connectable
where
    A: ResonixAudioNode + 'static,
{
    fn from(value: Audio<A>) -> Self {
        Connectable::AudioNode(Box::new(value.into_inner()))
    }
}

impl<P> From<Param<P>> for Connectable
where
    P: ResonixParamNode + 'static,
{
    fn from(value: Param<P>) -> Self {
        Connectable::ParamNode(Box::new(value.into_inner()))
    }
}
