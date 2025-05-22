use alloc::{boxed::Box, vec::Vec};

use crate::{
    Audio, DescribePorts, GetNodeId, Param, ResonixAudioNode, ResonixId, ResonixParamNode,
    ResonixPortAddress,
};

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

impl DescribePorts for Connectable {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        match self {
            Connectable::AudioNode(resonix_audio_node) => resonix_audio_node.input_port_addresses(),
            Connectable::ParamNode(resonix_param_node) => resonix_param_node.input_port_addresses(),
        }
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        match self {
            Connectable::AudioNode(resonix_audio_node) => {
                resonix_audio_node.output_port_addresses()
            }
            Connectable::ParamNode(resonix_param_node) => {
                resonix_param_node.output_port_addresses()
            }
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
