use alloc::boxed::Box;

use crate::{ResonixAudioNode, ResonixGraph, ResonixParamNode};

pub enum Connectable {
    AudioNode(Box<dyn ResonixAudioNode>),
    ParamNode(Box<dyn ResonixParamNode>),
    Graph(Box<dyn ResonixGraph>),
}
