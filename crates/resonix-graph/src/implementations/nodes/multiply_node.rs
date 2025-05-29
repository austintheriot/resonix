use core::ops::Deref;

use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::{
    errors::AudioNodeRunError,
    primitives::{Data, Id, NodeId, PortAddress, PortAddressDirection, PortId, Priority},
    traits::{
        Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority,
    },
};

pub struct MultiplyNode {
    node_id: NodeId,
    left_operand_value: Data,
    right_operand_value: Data,
    port_descriptors: MultiplyNodePortDescriptors,
}

impl MultiplyNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        Self::new_with_values(id_generator, Data::None, Data::None)
    }

    pub fn new_with_values<G: GenerateId, L: Into<Data>, R: Into<Data>>(
        id_generator: &mut G,
        left_operand: L,
        right_operand: R,
    ) -> Audio<Self> {
        let node_id = NodeId::from(id_generator.generate_id());
        let left_operand_value = left_operand.into();
        let right_operand_value = right_operand.into();
        let port_descriptors = MultiplyNodePortDescriptors::new(node_id);

        let multiply_node = Self {
            node_id,
            right_operand_value,
            left_operand_value,
            port_descriptors,
        };
        Audio(multiply_node)
    }

    fn assign_inputs(
        &mut self,
        inputs: &HashMap<PortAddress, &Data>,
    ) -> Result<(), AudioNodeRunError> {
        // TODO: validate inputs
        inputs.iter().for_each(|(port_address, data)| {
            let port_id = port_address.port_id();
            let left_port_id = self.left_operand_input_address().port_id();
            let right_port_id = self.right_operand_input_address().port_id();

            if port_id == left_port_id {
                self.left_operand_value = (**data).clone()
            } else if port_id == right_port_id {
                self.right_operand_value = (**data).clone()
            }
        });

        Ok(())
    }
}

impl GetPortDescriptors<MultiplyNodePortDescriptors> for MultiplyNode {
    fn get_port_descriptors(&self) -> MultiplyNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for MultiplyNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for MultiplyNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for MultiplyNode {
    fn run(
        &mut self,
        inputs: &HashMap<PortAddress, &Data>,
    ) -> Result<Option<HashMap<PortAddress, Data>>, AudioNodeRunError> {
        self.assign_inputs(inputs)?;

        let data = match self.left_operand_value {
            Data::F32(original_value_f32) => match self.right_operand_value {
                Data::F32(multiplier_f32) => Data::F32(original_value_f32 * multiplier_f32),
                Data::I32(multiplier_i32) => Data::F32(original_value_f32 * multiplier_i32 as f32),
                Data::None => Data::Error,
                Data::Error => Data::Error,
            },
            Data::I32(original_value_i32) => match self.right_operand_value {
                Data::F32(multiplier_f32) => Data::I32(original_value_i32 * multiplier_f32 as i32),
                Data::I32(multiplier_i32) => Data::I32(original_value_i32 * multiplier_i32),
                Data::None => Data::Error,
                Data::Error => Data::Error,
            },
            Data::None => Data::Error,
            Data::Error => Data::Error,
        };

        let outputs = HashMap::from([(self.output_port_address(), data)]);

        Ok(Some(outputs))
    }
}

impl Deref for MultiplyNode {
    type Target = MultiplyNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[derive(Copy, Clone)]
pub struct MultiplyNodePortDescriptors {
    node_id: NodeId,
}

impl MultiplyNodePortDescriptors {
    pub const LEFT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const RIGHT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(1usize);

    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    pub fn left_operand_input_address(&self) -> PortAddress {
        PortAddress::new(
            self.node_id,
            Self::LEFT_OPERAND_INPUT_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn right_operand_input_address(&self) -> PortAddress {
        PortAddress::new(
            self.node_id,
            Self::RIGHT_OPERAND_INPUT_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn output_port_address(&self) -> PortAddress {
        PortAddress::new(
            self.node_id,
            Self::OUTPUT_PORT_ID,
            PortAddressDirection::Output,
        )
    }
}

impl DescribePorts for MultiplyNodePortDescriptors {
    fn input_port_addresses(&self) -> Vec<PortAddress> {
        vec![self.left_operand_input_address()]
    }

    fn output_port_addresses(&self) -> Vec<PortAddress> {
        vec![self.output_port_address()]
    }

    fn param_port_addresses(&self) -> Vec<PortAddress> {
        vec![]
    }
}
