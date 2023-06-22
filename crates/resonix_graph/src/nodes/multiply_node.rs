use std::any::Any;

use uuid::Uuid;

use crate::{AudioContext, Connection, Node, NodeType, AddToContext};

/// Takes two signals and multiplies them together,
/// outputting the signal to all connected outputs
///
/// Input 0 - Signal 1
/// Input 1 - Signal 2
///
/// Output 0 - Multiplied signal
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MultiplyNode {
    uuid: Uuid,
}

impl MultiplyNode {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Node for MultiplyNode {
    fn node_type(&self) -> crate::NodeType {
        NodeType::Effect
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        inputs: &[&Connection],
        outputs: &mut [&mut Connection],
    ) {
        let first_input = inputs.get(0).unwrap();
        let second_input = inputs.get(1).unwrap();
        let result = first_input.data() * second_input.data();

        // copy to all output connections
        outputs.into_iter().for_each(|mut output| {
            output.set_data(result);
           
        })
    }

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("MultiplyNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AddToContext for MultiplyNode {}

impl Default for MultiplyNode {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4()
        }
    }
}

#[cfg(test)]
mod test_multiply_node {

    use uuid::Uuid;

    use crate::{AudioContext, Connection, MultiplyNode, Node};

    #[test]
    fn should_multiply_1st_and_2nd_inputs() {
        let mut multiply_node = MultiplyNode::new();

        let left_input_connection = Connection {
            from_index: 0,
            to_index: 0,
            data: 0.5,
            uuid: Uuid::new_v4(),
        };

        let right_input_connection = Connection {
            from_index: 0,
            to_index: 1,
            data: 0.2,
            uuid: Uuid::new_v4(),
        };

        let mut output_connection = Connection {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            uuid: Uuid::new_v4(),
        };

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [&left_input_connection, &right_input_connection];
            let mut outputs = [&mut output_connection];
            multiply_node.process(&inputs, &mut outputs)
        }

        // before processing, output data is 0.1
        {
            assert_eq!(output_connection.data(), 0.1);
        }
    }
}
