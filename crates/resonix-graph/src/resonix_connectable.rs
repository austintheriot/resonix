use alloc::{boxed::Box, vec::Vec};

use crate::{ResonixAudioNode, ResonixParamNode, ResonixPortAddress, resonix_audio_node};

pub enum Connectable {
    AudioNode(Box<dyn ResonixAudioNode>),
    ParamNode(Box<dyn ResonixParamNode>),
}

impl Connectable {
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

impl From<Box<dyn ResonixAudioNode>> for Connectable {
    fn from(value: Box<dyn ResonixAudioNode>) -> Self {
        Connectable::AudioNode(value)
    }
}

impl From<Box<dyn ResonixParamNode>> for Connectable {
    fn from(value: Box<dyn ResonixParamNode>) -> Self {
        Connectable::ParamNode(value)
    }
}
