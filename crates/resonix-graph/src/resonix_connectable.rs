use alloc::boxed::Box;

use crate::{ResonixAudioNode, ResonixParamNode};

pub enum Connectable {
    AudioNode(Box<dyn ResonixAudioNode>),
    ParamNode(Box<dyn ResonixParamNode>),
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
