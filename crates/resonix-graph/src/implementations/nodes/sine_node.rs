use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Deref;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        AudioNodeCtx, Id, NodeId, PortAddress, PortAddressDirection, PortDescriptor, PortId,
        Priority, Sample,
    },
    traits::{
        AudioBuffer, AudioBufferMut, AudioNode, DescribePorts, GenerateId, GetNodeId,
        GetPortDescriptors, GetPriority,
    },
};

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct SineNode {
    node_id: NodeId,
    frequencies: Box<[Sample]>,
    port_descriptors: SineNodePortDescriptors,
}

impl SineNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Self {
        Self::new_with_frequency(id_generator, Sample::default())
    }

    // TODO: replace these init functions with a dedicated builder interface
    pub fn new_with_frequency<G: GenerateId, F: Into<Sample>>(
        id_generator: &mut G,
        frequency: F,
    ) -> Self {
        Self::new_with_frequency_and_channels(id_generator, frequency, 1)
    }

    pub fn new_with_frequencies<G: GenerateId, const N: usize>(
        id_generator: &mut G,
        sine_values: [Sample; N],
    ) -> Self {
        let node_id = NodeId::from(id_generator.generate_id());
        let channels = sine_values.len();
        let port_descriptors = SineNodePortDescriptors::new(node_id, channels);
        let sine_values = Box::new(sine_values);

        Self {
            node_id,
            frequencies: sine_values,
            port_descriptors,
        }
    }

    pub fn new_with_frequency_and_channels<G: GenerateId, S: Into<Sample>>(
        id_generator: &mut G,
        sine_value: S,
        channels: usize,
    ) -> Self {
        let node_id = NodeId::from(id_generator.generate_id());
        let port_descriptors = SineNodePortDescriptors::new(node_id, channels);
        let mut frequencies = Vec::with_capacity(channels);
        frequencies.resize(channels, sine_value.into());

        Self {
            node_id,
            frequencies: frequencies.into_boxed_slice(),
            port_descriptors,
        }
    }
}

impl GetPortDescriptors<SineNodePortDescriptors> for SineNode {
    fn get_port_descriptors(&self) -> SineNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for SineNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for SineNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for SineNode {
    fn process<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        _inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        AudioNodeCtx {
            current_time,
            sample_rate,
            ..
        }: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError> {
        let output_port_slot = **SineNodePortDescriptors::OUTPUT_PORT_ID;

        let Some(output_buf) = outputs.get_mut(output_port_slot).and_then(|o| o.as_mut()) else {
            return Ok(());
        };

        // fill all samples in each channel with that channel's value
        for (i, channel) in output_buf.channels_iter_mut()?.enumerate() {
            // channel length match is checked at `connect` time
            let frequency = **self.frequencies.get(i).unwrap();

            for (sample_i, sample) in channel.iter_mut().enumerate() {
                // we must project ahead a bit per-sample based on
                // the sample rate to get correct values
                let sample_time = *current_time as f32 + sample_i as f32 / *sample_rate as f32;

                // output = A * sin(2PI * frequency * time)
                // where A is amplitude
                // frequency is number of oscillations per second
                // and time is the current time
                let value = f32::sin(2.0 * core::f32::consts::PI * frequency * sample_time);

                *sample = Sample::from(value);
            }
        }

        Ok(())
    }
}

impl Deref for SineNode {
    type Target = SineNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use approx::assert_relative_eq;

    use super::*;
    use crate::{
        primitives::{BlockSize, CurrentTime, Sample, SampleRate},
        test_utils::TestIdGenerator,
    };

    fn expected_sine_value(frequency: Sample, current_time: CurrentTime) -> Sample {
        Sample::from(f32::sin(
            core::f32::consts::PI * 2.0 * *frequency * (*current_time as f32),
        ))
    }

    /// Runs `node.process()` with no inputs and returns the output buffer contents.
    fn process_sine(
        node: &mut SineNode,
        block_size: BlockSize,
        current_time: CurrentTime,
    ) -> Vec<Sample> {
        let mut buf = vec![Sample::default(); *block_size];
        {
            let audio_buf_mut =
                crate::implementations::AudioBufferMut::new(buf.as_mut_slice(), 1).unwrap();

            let inputs: &[Option<crate::implementations::AudioBufferMut<'_>>] = &[];
            let mut outputs: Vec<Option<crate::implementations::AudioBufferMut<'_>>> =
                vec![Some(audio_buf_mut)];

            node.process(
                inputs,
                outputs.as_mut_slice(),
                AudioNodeCtx {
                    current_time,
                    block_size,
                    sample_rate: SampleRate::default(),
                },
            )
            .expect("process should not fail");
        }
        buf
    }

    // enables measuring output in increments of PI
    const DEFAULT_FREQUENCY: Sample = Sample::new(1.0);
    // enables measuring when the cycle is typically not at 0.0
    // for a frequency of 1 cycle per second, the value at t=0.25 seconds should be 1.0
    const DEFAULT_CURRENT_TIME: CurrentTime = CurrentTime::new(0.25);

    #[test]
    fn outputs_zero_when_frequency_of_0_is_chosen() {
        let mut id_gen = TestIdGenerator(0);
        let frequency = Sample::from(0.0);
        let mut node = SineNode::new_with_frequency(&mut id_gen, frequency);

        let result = process_sine(&mut node, BlockSize::from(1), DEFAULT_CURRENT_TIME);

        result
            .into_iter()
            .zip(vec![expected_sine_value(frequency, DEFAULT_CURRENT_TIME)])
            .for_each(|(result, expected)| assert_relative_eq!(*result, *expected));
    }

    #[test]
    fn outputs_expected_sine_value_for_single_sample() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = SineNode::new_with_frequencies(&mut id_gen, [DEFAULT_FREQUENCY]);

        let result = process_sine(&mut node, BlockSize::from(1), DEFAULT_CURRENT_TIME);

        result
            .into_iter()
            .zip(vec![expected_sine_value(
                DEFAULT_FREQUENCY,
                DEFAULT_CURRENT_TIME,
            )])
            .for_each(|(result, expected)| assert_relative_eq!(*result, *expected));
    }

    #[test]
    #[ignore]
    fn fills_entire_block_with_expected_sine_values() {
        let mut id_gen = TestIdGenerator(0);
        let frequencies = [
            Sample::new(0.0),
            Sample::new(1.0),
            Sample::new(3.0),
            Sample::new(4.0),
        ];
        let mut node = SineNode::new_with_frequencies(&mut id_gen, frequencies);

        let result = process_sine(&mut node, BlockSize::from(1), DEFAULT_CURRENT_TIME);

        result
            .into_iter()
            .zip(vec![
                expected_sine_value(frequencies[0], DEFAULT_CURRENT_TIME),
                expected_sine_value(frequencies[1], DEFAULT_CURRENT_TIME),
                expected_sine_value(frequencies[2], DEFAULT_CURRENT_TIME),
                expected_sine_value(frequencies[3], DEFAULT_CURRENT_TIME),
            ])
            .for_each(|(result, expected)| assert_relative_eq!(*result, *expected));
    }

    #[test]
    #[ignore]
    fn does_nothing_when_output_slot_is_not_connected() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = SineNode::new_with_frequency(&mut id_gen, DEFAULT_FREQUENCY);
        let inputs: &[Option<crate::implementations::AudioBufferMut<'_>>] = &[];
        let mut outputs: Vec<Option<crate::implementations::AudioBufferMut<'_>>> = vec![None];

        let result = node.process(
            inputs,
            outputs.as_mut_slice(),
            AudioNodeCtx {
                block_size: BlockSize::from(2048),
                current_time: DEFAULT_CURRENT_TIME,
                sample_rate: SampleRate::default(),
            },
        );

        assert!(result.is_ok());
    }
}

#[derive(Copy, Clone)]
pub struct SineNodePortDescriptors {
    node_id: NodeId,
    output_port_descriptors: [PortDescriptor; 1],
}

impl SineNodePortDescriptors {
    pub fn new(node_id: NodeId, channels: usize) -> Self {
        Self {
            node_id,
            output_port_descriptors: [PortDescriptor {
                address: Self::gen_output_port_address(node_id),
                channels,
            }],
        }
    }
}

impl DescribePorts for SineNodePortDescriptors {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        Some(&self.output_port_descriptors)
    }
}

impl SineNodePortDescriptors {
    // TODO: implement setting the sine values with audio-rate inputs
    pub const OUTPUT_PORT_ID: PortId = PortId::new(0usize);

    fn gen_output_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(node_id, Self::OUTPUT_PORT_ID, PortAddressDirection::Output)
    }

    pub fn output_port_address(&self) -> PortAddress {
        Self::gen_output_port_address(self.node_id)
    }
}
