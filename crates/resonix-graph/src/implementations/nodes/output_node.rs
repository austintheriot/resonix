use core::ops::Deref;

use crate::{
    errors::AudioNodeRunError,
    primitives::{Id, NodeId, PortAddress, PortAddressDirection, PortId, Priority, Sample},
    traits::{
        Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority,
    },
};

pub struct OutputNode {
    node_id: NodeId,
    input_value: Option<Sample>,
    port_descriptors: OutputNodePortDescriptors,
}

impl OutputNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        let node_id: NodeId = id_generator.generate_id().into();
        let output_node = Self {
            node_id,
            input_value: None,
            port_descriptors: OutputNodePortDescriptors::new(node_id),
        };
        Audio(output_node)
    }
}

impl GetPortDescriptors<OutputNodePortDescriptors> for OutputNode {
    fn get_port_descriptors(&self) -> OutputNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for OutputNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for OutputNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for OutputNode {
    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
    ) -> Result<(), AudioNodeRunError> {
        let input_port_id = **OutputNodePortDescriptors::INPUT_PORT_ID;
        let output_port_id = **self.external_output_port_address().port_id();

        if inputs.len() <= input_port_id || outputs.len() <= output_port_id {
            return Ok(());
        }

        let input_block = inputs[input_port_id];
        let output_block = &mut outputs[output_port_id];

        for (out, inp) in output_block.iter_mut().zip(input_block.iter()) {
            *out = *inp;
        }

        self.input_value = input_block.last().copied();

        Ok(())
    }
}

impl Deref for OutputNode {
    type Target = OutputNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[derive(Copy, Clone)]
pub struct OutputNodePortDescriptors {
    node_id: NodeId,
    input_port_addresses: [PortAddress; 1],
    external_port_address: [PortAddress; 1],
}

impl DescribePorts for OutputNodePortDescriptors {
    fn input_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.input_port_addresses)
    }

    fn external_output_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.external_port_address)
    }
}

impl OutputNodePortDescriptors {
    pub const INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const EXTERNAL_OUTPUT_PORT_ID: PortId = PortId::new(1usize);

    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            input_port_addresses: [Self::gen_input_port_address(node_id)],
            external_port_address: [Self::gen_external_output_port_address(node_id)],
        }
    }

    fn gen_input_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(node_id, Self::INPUT_PORT_ID, PortAddressDirection::Input)
    }

    fn gen_external_output_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::EXTERNAL_OUTPUT_PORT_ID,
            PortAddressDirection::ExternalOutput,
        )
    }

    pub fn input_port_address(&self) -> PortAddress {
        Self::gen_input_port_address(self.node_id)
    }

    pub fn external_output_port_address(&self) -> PortAddress {
        Self::gen_external_output_port_address(self.node_id)
    }
}
