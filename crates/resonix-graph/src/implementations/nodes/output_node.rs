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

pub struct OutputNode {
    node_id: NodeId,
    input_value: Data,
    port_descriptors: OutputNodePortDescriptors,
}

impl OutputNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        let node_id: NodeId = id_generator.generate_id().into();
        let output_node = Self {
            node_id,
            input_value: Data::None,
            port_descriptors: OutputNodePortDescriptors::new(node_id),
        };
        Audio(output_node)
    }

    fn assign_inputs(
        &mut self,
        inputs: &HashMap<PortAddress, &Data>,
    ) -> Result<(), AudioNodeRunError> {
        let num_inputs = inputs.len();
        if num_inputs > 1 {
            // TODO: lift this requirement?
            return Err(AudioNodeRunError::TooManyInputs {
                expected: 1,
                found: num_inputs,
            });
        }

        if inputs.is_empty() {
            return Ok(());
        }

        // TODO: return error if port doesn't match
        self.input_value = (*inputs.get(&self.input_port_address()).unwrap()).clone();

        Ok(())
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
        inputs: &HashMap<PortAddress, &Data>,
    ) -> Result<Option<HashMap<PortAddress, Data>>, AudioNodeRunError> {
        self.assign_inputs(inputs)?;

        // nothing to do--just receives input
        Ok(None)
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
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
