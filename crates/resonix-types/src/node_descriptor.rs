use crate::{AudioContextUid, AudioPortDescriptor};

/// Contains stateless details about the Node,
/// such as its uid, name, how many input ports it has,
/// how many output ports it has, etc.
pub trait NodeDescriptor {
    fn node_uid(&self) -> &impl AudioContextUid;

    /// Human-readable name
    fn name(&self) -> &'static str;

    /// Human-readable description
    fn description(&self) -> &'static str;

    fn num_input_ports(&self) -> u8;

    fn num_output_ports(&self) -> u8;

    fn input_ports(&self) -> &impl Iterator<Item = impl AudioPortDescriptor>;

    fn output_ports(&self) -> &impl Iterator<Item = impl NodeDescriptor>;
}
