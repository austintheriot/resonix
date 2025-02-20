use crate::{AudioContext, ConnectionDescriptor, NodeDescriptor};

pub trait Node {
    type Options;

    fn new(audio_context: &mut impl AudioContext) -> impl Node;

    fn new_with_options(audio_context: &mut impl AudioContext, options: Self::Options)
        -> impl Node;

    fn descriptor(&self) -> &impl NodeDescriptor;

    fn current_input_connections(&self)
        -> Option<&impl Iterator<Item = impl ConnectionDescriptor>>;

    fn current_output_connections(
        &self,
    ) -> Option<&impl Iterator<Item = impl ConnectionDescriptor>>;
}
