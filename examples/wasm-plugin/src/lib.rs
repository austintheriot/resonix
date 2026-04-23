#![no_std]

use resonix_core::{
    errors::AudioNodeRunError,
    primitives::{
        AudioNodeCtx, Id, NodeId, PortAddress, PortAddressDirection, PortDescriptor, PortId,
        Priority, Sample,
    },
    traits::{
        AudioBuffer, AudioBufferMut, DescribePorts, GetNodeId, GetPortDescriptors, GetPriority,
    },
};
use resonix_wasm_audio_node::wasm_audio_node;

const CHANNELS: usize = 2;

struct ExamplePorts {
    inputs: [PortDescriptor; 1],
    outputs: [PortDescriptor; 1],
}

impl DescribePorts for ExamplePorts {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        Some(&self.inputs)
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        Some(&self.outputs)
    }
}

#[wasm_audio_node(init = Example::new())]
struct Example;

impl Example {
    pub fn new() -> Self {
        Example
    }
}

// TODO: must remove `GetNodeId` from AudioNode trait
// This is ignored by the host
impl GetNodeId for Example {
    fn node_id(&self) -> Id {
        Id::new(0)
    }
}

// TODO: must remove `GetNodeId` from AudioNode trait
// This is ignored by the host
impl GetPriority for Example {
    fn get_priority(&self) -> Priority {
        Priority::new(0)
    }
}

impl DescribePorts for Example {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }
}

impl GetPortDescriptors<ExamplePorts> for Example {
    fn get_port_descriptors(&self) -> ExamplePorts {
        ExamplePorts {
            inputs: [PortDescriptor {
                address: PortAddress::new(
                    NodeId::new(0),
                    PortId::new(0),
                    PortAddressDirection::Input,
                ),
                channels: CHANNELS,
            }],
            outputs: [PortDescriptor {
                address: PortAddress::new(
                    NodeId::new(0),
                    PortId::new(0),
                    PortAddressDirection::Output,
                ),
                channels: CHANNELS,
            }],
        }
    }
}

impl resonix_core::traits::AudioNode for Example {
    fn process<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        _ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError> {
        let Some(input) = inputs.get(0).and_then(|o| o.as_ref()) else {
            return Ok(());
        };
        let Some(output) = outputs.get_mut(0).and_then(|o| o.as_mut()) else {
            return Ok(());
        };

        for ch in 0..CHANNELS {
            let in_ch = input.channel(ch)?;
            let out_ch = output.channel_mut(ch)?;
            for (o, i) in out_ch.iter_mut().zip(in_ch.iter()) {
                *o = Sample::from(**i * 0.5);
            }
        }

        Ok(())
    }
}
