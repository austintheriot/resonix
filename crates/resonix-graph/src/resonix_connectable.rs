use alloc::boxed::Box;

use crate::{ResonixAudioNode, ResonixParamNode};

pub enum Connectable {
    AudioNode(Box<dyn ResonixAudioNode>),
    ParamNode(Box<dyn ResonixParamNode>),
}
