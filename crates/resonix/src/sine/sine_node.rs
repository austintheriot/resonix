use std::{cell::{Ref, RefMut, RefCell}, rc::Rc};

use uuid::Uuid;

use crate::{Node, Sine, Connection, SineInterface, SampleRate, NodeType, AudioContext, Connect};

#[derive(Debug, Clone, Default)]
pub struct SineNode {
    uuid: Uuid,
    sine: Rc<RefCell<Sine>>,
    audio_context: AudioContext,
}


impl SineInterface for SineNode {
    fn next_sample(&mut self) -> f32 {
        self.sine.borrow_mut().next_sample()
    }

    fn set_sample_rate(&mut self, sample_rate: impl Into<SampleRate>) -> &mut Self {
        self.sine.borrow_mut().set_sample_rate(sample_rate);
        self
    }

    fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.sine.borrow_mut().set_frequency(frequency);
        self
    }

    fn sample_rate(&self) -> SampleRate {
        self.sine.borrow().sample_rate()
    }

    fn frequency(&self) -> f32 {
        self.sine.borrow().frequency()
    }
}

impl Node for SineNode {
    fn process(&mut self, _inputs: &[Ref<Connection>], outputs: &mut [RefMut<Connection>]) {
        let next_sample = self.next_sample();
        outputs.iter_mut().for_each(|output| {
            output.data = next_sample;
        });
    }

    fn node_type(&self) -> NodeType {
        NodeType::Input
    }
    
    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl Connect for SineNode {
    fn connect_nodes_with_indexes<N: Node + Connect + Clone>(&self, from_index: usize, other_node: &N, to_index: usize) -> &Self {
        self.audio_context.connect_nodes_with_indexes(self.clone(), from_index, other_node.clone(), to_index);
        self
    }
}

impl SineNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_sine_node = Self {
            uuid: Uuid::new_v4(),
            sine: Rc::new(RefCell::new(Sine::new())),
            audio_context: audio_context.clone(),
        };

        audio_context.add_node(RefCell::new(Box::new(new_sine_node.clone())));

        new_sine_node
    }

    pub fn new_with_config(audio_context: &mut AudioContext, sample_rate: impl Into<SampleRate>, frequency: impl Into<f32>) -> Self {
        let new_sine_node = Self {
            uuid: Uuid::new_v4(),
            sine: Rc::new(RefCell::new(Sine::new_with_config(sample_rate, frequency))),
            audio_context: audio_context.clone(),
        };

        audio_context.add_node(RefCell::new(Box::new(new_sine_node.clone())));

        new_sine_node
    }
}

impl PartialEq for SineNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for SineNode {}