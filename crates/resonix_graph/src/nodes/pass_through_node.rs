use std::any::Any;

use uuid::Uuid;

use crate::{AddToContext, Connection, Node, NodeType};

/// Takes one signal and passed it through, unaltered
/// to all connected outputs.
///
/// Input 0 - Input signal
///
/// Output 0 - Unaltered Input signal
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PassThroughNode {
    uuid: Uuid,
}

impl PassThroughNode {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Node for PassThroughNode {
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        let input_data = inputs.next().map(|c| c.data()).unwrap_or(0.0);

        // copy first input to all output connections
        for output in outputs.into_iter() {
            output.set_data(input_data);
        }
    }

    fn node_type(&self) -> NodeType {
        NodeType::Effect
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("PassThroughNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AddToContext for PassThroughNode {}

impl Default for PassThroughNode {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

#[cfg(test)]
mod test_pass_through_node {

    use uuid::Uuid;

    use crate::{Connection, Node, PassThroughNode};

    #[test]
    fn should_pass_audio_data_through_output_connections() {
        let mut pass_through_node = PassThroughNode::new();

        let input_connection = Connection {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            uuid: Uuid::new_v4(),
        };

        let mut output_connection = Connection {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            uuid: Uuid::new_v4(),
        };

        // before processing, output connection holds 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
        }

        {
            let inputs = [&input_connection];
            let outputs = [&mut output_connection];
            pass_through_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output connection holds input data
        {
            assert_eq!(output_connection.data(), 0.1234);
        }
    }
}
