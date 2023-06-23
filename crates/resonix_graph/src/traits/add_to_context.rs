use async_trait::async_trait;
use petgraph::stable_graph::NodeIndex;

use crate::{AudioContext, Node, ProcessorInterface};

#[async_trait]
pub trait AddToContext
where
    Self: Node + Sized + 'static,
{
    async fn add_to_context(self, audio_context: &mut AudioContext) -> Result<NodeIndex, Self> {
       audio_context.add_node(self).await
    }
}
