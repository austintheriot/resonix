use petgraph::stable_graph::NodeIndex;

use crate::{AudioContext, Node};

pub trait AddToContext
where
    Self: Node + Sized + 'static,
{
    fn add_to_context(self, audio_context: &mut AudioContext) -> Result<NodeIndex, Self> {
        audio_context.add_node(self)
    }
}
