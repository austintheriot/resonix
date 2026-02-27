use alloc::boxed::Box;

use super::{AudioBuffer, BlockSize, PortId};

/// Short-lived struct that eases passing data into the `AudioNode::process` call
#[derive(Debug, PartialEq)]
pub struct NodeProcessContext<'a> {
    inputs: Box<[Option<&'a AudioBuffer>]>,
    outputs: Box<[Option<&'a mut AudioBuffer>]>,
    block_size: BlockSize,
}

impl<'a> NodeProcessContext<'a> {
    pub fn new(block_size: BlockSize) -> Self {
        Self {
            inputs: Box::new([None; PortId::MAX_PORT_ID]),
            outputs: Box::new([const { None }; PortId::MAX_PORT_ID]),
            block_size,
        }
    }

    pub fn input_buffer(&self, port_id: impl Into<PortId>) -> Option<&AudioBuffer> {
        self.inputs
            .get(**port_id.into())
            .and_then(|inner: &Option<&Box<[f32]>>| inner.as_deref())
    }

    pub fn output_buffer(&mut self, port_id: impl Into<PortId>) -> Option<&mut AudioBuffer> {
        self.outputs
            .get_mut(**port_id.into())
            .and_then(|inner: &mut Option<&mut Box<[f32]>>| inner.as_deref_mut())
    }

    pub fn set_input_buffer<'b: 'a>(&mut self, port_id: PortId, input_buffer: &'b Box<[f32]>) {
        self.inputs[**port_id] = Some(input_buffer);
    }

    pub fn set_output_buffer<'b: 'a>(
        &mut self,
        port_id: PortId,
        output_buffer: &'b mut Box<[f32]>,
    ) {
        self.outputs[**port_id] = Some(output_buffer);
    }

    pub fn block_size(&self) -> BlockSize {
        self.block_size
    }
}
