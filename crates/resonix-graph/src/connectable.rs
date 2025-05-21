use alloc::{boxed::Box, vec::Vec};

use crate::{Audio, Param, ResonixAudioNode, ResonixId, ResonixParamNode, ResonixPortAddress};

pub enum Connectable {
    AudioNode(Box<dyn ResonixAudioNode>),
    ParamNode(Box<dyn ResonixParamNode>),
}

impl Connectable {
    pub fn node_id(&self) -> ResonixId {
        match self {
            Connectable::AudioNode(resonix_audio_node) => resonix_audio_node.node_id(),
            Connectable::ParamNode(resonix_param_node) => resonix_param_node.node_id(),
        }
    }

    pub fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        match self {
            Connectable::AudioNode(resonix_audio_node) => resonix_audio_node.input_port_addresses(),
            Connectable::ParamNode(resonix_param_node) => resonix_param_node.input_port_addresses(),
        }
    }

    pub fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
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

impl<T: ResonixAudioNode + 'static> From<Audio<T>> for Connectable {
    fn from(value: Audio<T>) -> Self {
        Connectable::AudioNode(Box::new(value.0))
    }
}

impl<T: ResonixParamNode + 'static> From<Param<T>> for Connectable {
    fn from(value: Param<T>) -> Self {
        Connectable::ParamNode(Box::new(value.0))
    }
}
