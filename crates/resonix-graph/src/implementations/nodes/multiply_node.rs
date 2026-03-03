use core::ops::Deref;

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        BlockSize, Id, NodeId, PortAddress, PortAddressDirection, PortId, Priority, Sample,
    },
    traits::{
        Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority,
    },
};

pub struct MultiplyNode {
    node_id: NodeId,
    left_operand_value: Sample,
    right_operand_value: Sample,
    port_descriptors: MultiplyNodePortDescriptors,
}

impl MultiplyNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        Self::new_with_values(id_generator, Sample::default(), Sample::default())
    }

    pub fn new_with_values<G: GenerateId, L: Into<Sample>, R: Into<Sample>>(
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
    fn process(
        &mut self,
        inputs: &[Option<&[Sample]>],
        outputs: &mut [Option<&mut [Sample]>],
        _block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError> {
        let left_port_slot = **MultiplyNodePortDescriptors::LEFT_OPERAND_INPUT_PORT_ID;
        let right_port_slot = **MultiplyNodePortDescriptors::RIGHT_OPERAND_INPUT_PORT_ID;
        let output_port_slot = **MultiplyNodePortDescriptors::OUTPUT_PORT_ID;

        let Some(output_block) = outputs[output_port_slot].as_deref_mut() else {
            return Ok(());
        };

        // TODO: handle None case (self-reference or not connected)
        let left_block = inputs[left_port_slot].unwrap_or(&[]);
        let right_block = inputs[right_port_slot].unwrap_or(&[]);

        for (i, out) in output_block.iter_mut().enumerate() {
            let left = left_block
                .get(i)
                .copied()
                .unwrap_or(self.left_operand_value);
            let right = right_block
                .get(i)
                .copied()
                .unwrap_or(self.right_operand_value);
            *out = Sample::new(*left * *right);
        }

        if let Some(last) = left_block.last() {
            self.left_operand_value = *last;
        }
        if let Some(last) = right_block.last() {
            self.right_operand_value = *last;
        }

        Ok(())
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
    input_port_addresses: [PortAddress; 2],
    output_port_addresses: [PortAddress; 1],
}

impl MultiplyNodePortDescriptors {
    pub const LEFT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const RIGHT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(1usize);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(0usize);

    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            input_port_addresses: [
                Self::gen_left_operand_input_address(node_id),
                Self::gen_right_operand_input_address(node_id),
            ],
            output_port_addresses: [Self::gen_output_port_address(node_id)],
        }
    }

    fn gen_left_operand_input_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::LEFT_OPERAND_INPUT_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn left_operand_input_address(&self) -> PortAddress {
        Self::gen_left_operand_input_address(self.node_id)
    }

    fn gen_right_operand_input_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::RIGHT_OPERAND_INPUT_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn right_operand_input_address(&self) -> PortAddress {
        Self::gen_right_operand_input_address(self.node_id)
    }

    fn gen_output_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(node_id, Self::OUTPUT_PORT_ID, PortAddressDirection::Output)
    }

    pub fn output_port_address(&self) -> PortAddress {
        Self::gen_output_port_address(self.node_id)
    }
}

impl DescribePorts for MultiplyNodePortDescriptors {
    fn input_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.input_port_addresses)
    }

    fn output_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.output_port_addresses)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::{
        primitives::{BlockSize, Sample},
        test_utils::TestIdGenerator,
    };

    /// Runs `node.process()` and returns the output buffer contents.
    /// `left` and `right` are the two input slots; `None` means unconnected.
    fn process_multiply(
        node: &mut MultiplyNode,
        left: Option<&[Sample]>,
        right: Option<&[Sample]>,
        block_size: usize,
    ) -> Vec<Sample> {
        let mut out_buf = vec![Sample::default(); block_size];
        let inputs: Vec<Option<&[Sample]>> = vec![left, right];
        {
            let mut outputs: Vec<Option<&mut [Sample]>> = vec![Some(out_buf.as_mut_slice())];
            node.process(
                inputs.as_slice(),
                outputs.as_mut_slice(),
                BlockSize::new(block_size),
            )
            .expect("process should not fail");
        }
        out_buf
    }

    #[test]
    fn multiplies_two_input_signals() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new(&mut id_gen).into_inner();
        let left = [Sample::from(2.0f32)];
        let right = [Sample::from(3.0f32)];
        let result = process_multiply(&mut node, Some(&left), Some(&right), 1);
        assert_eq!(result, vec![Sample::from(6.0f32)]);
    }

    #[test]
    fn defaults_to_zero_when_no_inputs_connected() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new(&mut id_gen).into_inner();
        // Both held values default to 0.0, so 0.0 * 0.0 = 0.0
        let result = process_multiply(&mut node, None, None, 1);
        assert_eq!(result, vec![Sample::from(0.0f32)]);
    }

    #[test]
    fn uses_initial_values_when_inputs_absent() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new_with_values(&mut id_gen, 4.0f32, 5.0f32).into_inner();
        // No live inputs — falls back to the initial held values: 4.0 * 5.0 = 20.0
        let result = process_multiply(&mut node, None, None, 1);
        assert_eq!(result, vec![Sample::from(20.0f32)]);
    }

    #[test]
    fn uses_held_value_for_missing_right_input() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new_with_values(&mut id_gen, 1.0f32, 3.0f32).into_inner();
        let left = [Sample::from(5.0f32)];
        // right not connected — uses held initial value of 3.0; result = 5.0 * 3.0 = 15.0
        let result = process_multiply(&mut node, Some(&left), None, 1);
        assert_eq!(result, vec![Sample::from(15.0f32)]);
    }

    #[test]
    fn uses_held_value_for_missing_left_input() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new_with_values(&mut id_gen, 3.0f32, 1.0f32).into_inner();
        let right = [Sample::from(5.0f32)];
        // left not connected — uses held initial value of 3.0; result = 3.0 * 5.0 = 15.0
        let result = process_multiply(&mut node, None, Some(&right), 1);
        assert_eq!(result, vec![Sample::from(15.0f32)]);
    }

    #[test]
    fn multiplies_block_element_wise() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new(&mut id_gen).into_inner();
        let left = [
            Sample::from(1.0f32),
            Sample::from(2.0f32),
            Sample::from(3.0f32),
        ];
        let right = [
            Sample::from(4.0f32),
            Sample::from(5.0f32),
            Sample::from(6.0f32),
        ];
        let result = process_multiply(&mut node, Some(&left), Some(&right), 3);
        assert_eq!(
            result,
            vec![
                Sample::from(4.0f32),
                Sample::from(10.0f32),
                Sample::from(18.0f32),
            ]
        );
    }

    #[test]
    fn updates_held_values_after_block() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new(&mut id_gen).into_inner();
        // First block sets last-seen values to 5.0 and 7.0
        let first_left = [Sample::from(5.0f32)];
        let first_right = [Sample::from(7.0f32)];
        process_multiply(&mut node, Some(&first_left), Some(&first_right), 1);
        // Second block with no live inputs uses held values: 5.0 * 7.0 = 35.0
        let result = process_multiply(&mut node, None, None, 1);
        assert_eq!(result, vec![Sample::from(35.0f32)]);
    }

    #[test]
    fn does_nothing_when_output_slot_is_not_connected() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = MultiplyNode::new(&mut id_gen).into_inner();
        let left = [Sample::from(5.0f32)];
        let right = [Sample::from(5.0f32)];
        let inputs: Vec<Option<&[Sample]>> = vec![Some(&left), Some(&right)];
        let mut outputs: Vec<Option<&mut [Sample]>> = vec![None];
        let result = node.process(inputs.as_slice(), outputs.as_mut_slice(), BlockSize::new(1));
        assert!(result.is_ok());
    }
}
