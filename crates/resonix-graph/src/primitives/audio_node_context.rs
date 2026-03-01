use super::{BlockSize, PortId, Sample};

/// Short-lived struct that eases passing data into the `AudioNode::process` call
#[derive(Debug, PartialEq)]
pub struct AudioNodeContext<'a> {
    /// Each slot in the input array represents the data block for that port.
    /// Each array is the audio data for the audio block current being processed.
    pub inputs: [Option<&'a [Sample]>; PortId::MAX_PORT_ID],
    /// Each slot in the output array represents the data block for that port
    /// Each array is the audio data for the audio block current being processed.
    pub outputs: [Option<&'a mut [Sample]>; PortId::MAX_PORT_ID],
    pub block_size: BlockSize,
}

impl<'a> AudioNodeContext<'a> {
    pub fn set_output_buffer(
        &mut self,
        port_id: impl Into<PortId>,
        output: Option<&'a mut [Sample]>,
    ) -> &mut Self {
        self.outputs[**port_id.into()] = output;
        self
    }

    pub fn set_input_buffer(
        &mut self,
        port_id: impl Into<PortId>,
        input: Option<&'a [Sample]>,
    ) -> &mut Self {
        self.inputs[**port_id.into()] = input;
        self
    }

    pub fn set_output_buffers(
        &mut self,
        outputs: [Option<&'a mut [Sample]>; PortId::MAX_PORT_ID],
    ) -> &mut Self {
        self.outputs = outputs;
        self
    }

    pub fn set_input_buffers(
        &mut self,
        inputs: [Option<&'a [Sample]>; PortId::MAX_PORT_ID],
    ) -> &mut Self {
        self.inputs = inputs;
        self
    }

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
