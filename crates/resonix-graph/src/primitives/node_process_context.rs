
use super::{BlockSize, PortId, Sample};

/// Short-lived struct that eases passing data into the `AudioNode::process` call
#[derive(Debug, PartialEq)]
pub struct NodeProcessContext<'a> {
    pub inputs: [Option<&'a [Sample]>; PortId::MAX_PORT_ID],
    pub outputs: [Option<&'a mut [Sample]>; PortId::MAX_PORT_ID],
    pub block_size: BlockSize,
}

impl<'a> NodeProcessContext<'a> {
    pub fn input_buffer(&self, port_id: impl Into<PortId>) -> Option<&[Sample]> {
        self.inputs
            .get(**port_id.into())
            .and_then(|inner: &Option<&[Sample]>| inner.as_deref())
    }

    pub fn output_buffer(&mut self, port_id: impl Into<PortId>) -> Option<&mut [Sample]> {
        self.outputs
            .get_mut(**port_id.into())
            .and_then(|inner: &mut Option<&mut [Sample]>| inner.as_deref_mut())
    }

    pub fn block_size(&self) -> BlockSize {
        self.block_size
    }
}
