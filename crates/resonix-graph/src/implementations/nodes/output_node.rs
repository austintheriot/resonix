use core::ops::Deref;

use alloc::vec::Vec;

use crate::{
    errors::AudioNodeAssignInputError,
    primitives::{Data, DataList, Id, NodeId, PortAddress, PortAddressDirection, PortId, Priority},
    traits::{
        Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority,
    },
};

pub struct OutputNode {
    node_id: NodeId,
    intput_value: Data,
    port_descriptors: OutputNodePortDescriptors,
}

impl OutputNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        let node_id: NodeId = id_generator.generate_id().into();
        let output_node = Self {
            node_id,
            intput_value: Data::None,
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
    fn run(&mut self) -> DataList {
        // nothing to do--just receives input
        // TODO: make output data `Option<DataList>`?
        DataList::empty()
    }

    fn assign_inputs(&mut self, mut inputs: DataList) -> Result<(), AudioNodeAssignInputError> {
        // TODO: lift this requirement? Sum inputs instead of throwing error?
        let num_inputs = inputs.len();
        if num_inputs > 1 {
            return Err(AudioNodeAssignInputError::TooManyInputs {
                expected: 1,
                found: num_inputs,
            });
        }

        self.intput_value = inputs.remove(0);

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
}

impl OutputNodePortDescriptors {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }
}

impl DescribePorts for OutputNodePortDescriptors {
    fn input_port_addresses(&self) -> Vec<PortAddress> {
        vec![self.input_port_address()]
    }

    fn output_port_addresses(&self) -> Vec<PortAddress> {
        vec![]
    }

    fn param_port_addresses(&self) -> Vec<PortAddress> {
        vec![]
    }
}

impl OutputNodePortDescriptors {
    pub const INPUT_PORT_ID: PortId = PortId::new(0usize);

    pub fn input_port_address(&self) -> PortAddress {
        PortAddress::new(
            self.node_id,
            Self::INPUT_PORT_ID,
            PortAddressDirection::Input,
        )
    }
}
