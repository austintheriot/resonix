use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{error::Error, ops::Deref};

use thiserror::Error;
use wasmer::{
    CompileError, ExportError, Instance, InstantiationError, Memory, Module, RuntimeError, Store,
    TypedFunction, imports, sys::instance,
};

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        AudioNodeCtx, Id, NodeId, PortAddress, PortAddressDirection, PortDescriptor, PortId,
        Priority,
    },
    traits::{
        AudioBuffer, AudioBufferMut, AudioNode, DescribePorts, GenerateId, GetNodeId,
        GetPortDescriptors, GetPriority,
    },
};

/// Per-port metadata cached at instantiation time.
struct PortInfo {
    channel_count: usize,
    /// One WASM linear memory byte offset per channel.
    channel_offsets: Box<[u64]>,
}

struct PortInfoData {
    input_ports: Box<[PortInfo]>,
    output_ports: Box<[PortInfo]>,
}

pub struct WasmNode {
    node_id: NodeId,
    store: Store,
    instance: Instance,
    port_descriptors: WasmNodePortDescriptors,
    port_info_data: PortInfoData,
    /// Cached handle — avoids repeated export lookup on the hot path.
    process_fn: TypedFunction<(i32, f64), ()>,
}

#[derive(Error, Debug)]
pub enum WasmNodeCreationError {
    #[error("wasm node encountered error while compiling: {0:?}")]
    CompileError(#[from] CompileError),
    #[error("wasm node encountered error while instantiating: {0:?}")]
    InstantiationError(#[from] InstantiationError),
    #[error("wasm node missing required export: {0}")]
    MissingExport(#[from] ExportError),
    #[error("wasm node encountered a runtime error while querying exports: {0:?}")]
    RuntimeError(#[from] RuntimeError),
}

impl WasmNode {
    pub fn new<G: GenerateId>(
        id_generator: &mut G,
        bytes: &[u8],
    ) -> Result<Self, WasmNodeCreationError> {
        let mut store = Store::default();
        let module = Module::new(&store, bytes)?;
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;
        let node_id = NodeId::from(id_generator.generate_id());
        let port_info_data = Self::build_port_info(&mut store, &instance)?;
        let port_descriptors = WasmNodePortDescriptors::new(
            node_id,
            &port_info_data.input_ports,
            &port_info_data.output_ports,
        );
        let process_fn: TypedFunction<(i32, f64), ()> = instance
            .exports
            .get_typed_function(&store, "resonix_process")?;

        Ok(Self {
            node_id,
            store,
            instance,
            port_descriptors,
            port_info_data,
            process_fn,
        })
    }

    fn build_port_info(
        mut store: &mut Store,
        instance: &Instance,
    ) -> Result<PortInfoData, WasmNodeCreationError> {
        let get_input_count: TypedFunction<(), i32> = instance
            .exports
            .get_typed_function(&store, "resonix_input_count")?;

        let get_output_count: TypedFunction<(), i32> = instance
            .exports
            .get_typed_function(&store, "resonix_output_count")?;

        let get_input_channel_count: TypedFunction<i32, i32> = instance
            .exports
            .get_typed_function(&store, "get_input_channel_count")?;

        let get_output_channel_count: TypedFunction<i32, i32> = instance
            .exports
            .get_typed_function(&store, "get_output_channel_count")?;

        let get_input_buffer_ptr: TypedFunction<(i32, i32), i32> = instance
            .exports
            .get_typed_function(&store, "get_input_buffer_ptr")?;

        let get_output_buffer_ptr: TypedFunction<(i32, i32), i32> = instance
            .exports
            .get_typed_function(&store, "get_output_buffer_ptr")?;

        let input_count = get_input_count.call(&mut store)? as usize;
        let output_count = get_output_count.call(&mut store)? as usize;

        let mut input_ports = Vec::with_capacity(input_count);
        for port in 0..input_count {
            let channel_count = get_input_channel_count.call(&mut store, port as i32)? as usize;
            let mut channel_offsets = Vec::with_capacity(channel_count);
            for ch in 0..channel_count {
                let offset = get_input_buffer_ptr.call(&mut store, port as i32, ch as i32)? as u64;
                channel_offsets.push(offset);
            }
            input_ports.push(PortInfo {
                channel_count,
                channel_offsets: channel_offsets.into_boxed_slice(),
            });
        }

        let mut output_ports = Vec::with_capacity(output_count);
        for port in 0..output_count {
            let channel_count = get_output_channel_count.call(&mut store, port as i32)? as usize;
            let mut channel_offsets = Vec::with_capacity(channel_count);
            for ch in 0..channel_count {
                let offset = get_output_buffer_ptr.call(&mut store, port as i32, ch as i32)? as u64;
                channel_offsets.push(offset);
            }
            output_ports.push(PortInfo {
                channel_count,
                channel_offsets: channel_offsets.into_boxed_slice(),
            });
        }

        Ok(PortInfoData {
            input_ports: input_ports.into_boxed_slice(),
            output_ports: output_ports.into_boxed_slice(),
        })
    }

    fn copy_inputs_into_wasm_linear_memory<A: AudioBuffer>(
        &mut self,
        inputs: &[Option<A>],
    ) -> Result<(), AudioNodeRunError> {
        let memory = self
            .memory()
            .map_err(|e| AudioNodeRunError::Unknown(Box::new(e)))?;

        for (port_idx, port_info) in self.port_info_data.input_ports.iter().enumerate() {
            let Some(Some(buf)) = inputs.get(port_idx) else {
                continue;
            };
            let view = memory.view(&self.store);
            let channels = buf.channels().min(port_info.channel_count);
            for ch in 0..channels {
                let samples = buf.channel(ch).map_err(|e| Box::new(e) as Box<dyn Error>)?;
                let byte_offset = port_info.channel_offsets[ch];
                // SAFETY: Sample is #[repr(transparent)] over f32.
                let bytes: &[u8] = unsafe {
                    core::slice::from_raw_parts(samples.as_ptr().cast::<u8>(), samples.len() * 4)
                };
                view.write(byte_offset, bytes)
                    .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            }
        }

        Ok(())
    }

    fn copy_outputs_out_of_wasm_linear_memory<M: AudioBufferMut>(
        &mut self,
        outputs: &mut [Option<M>],
    ) -> Result<(), AudioNodeRunError> {
        let memory = self.memory().map_err(|e| Box::new(e) as Box<dyn Error>)?;

        for (port_idx, port_info) in self.port_info_data.output_ports.iter().enumerate() {
            let Some(Some(buf)) = outputs.get_mut(port_idx) else {
                continue;
            };
            let view = memory.view(&self.store);
            let channels = buf.channels().min(port_info.channel_count);
            for ch in 0..channels {
                let out_slice = buf
                    .channel_mut(ch)
                    .map_err(|e| Box::new(e) as Box<dyn Error>)?;
                let byte_offset = port_info.channel_offsets[ch];
                // SAFETY: Sample is #[repr(transparent)] over f32.
                let bytes: &mut [u8] = unsafe {
                    core::slice::from_raw_parts_mut(
                        out_slice.as_mut_ptr().cast::<u8>(),
                        out_slice.len() * 4,
                    )
                };
                view.read(byte_offset, bytes)
                    .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            }
        }

        Ok(())
    }

    fn memory(&self) -> Result<&Memory, ExportError> {
        self.instance.exports.get_memory("memory")
    }
}

impl GetPortDescriptors<WasmNodePortDescriptors> for WasmNode {
    fn get_port_descriptors(&self) -> WasmNodePortDescriptors {
        self.port_descriptors.clone()
    }
}

impl GetNodeId for WasmNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

impl GetPriority for WasmNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for WasmNode {
    fn process<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError> {
        let block_size = *ctx.block_size as i32;
        let current_time = *ctx.current_time;

        self.copy_inputs_into_wasm_linear_memory(inputs)?;

        self.process_fn
            .call(&mut self.store, block_size, current_time)
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;

        self.copy_outputs_out_of_wasm_linear_memory(outputs)?;

        Ok(())
    }
}

impl Deref for WasmNode {
    type Target = WasmNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[derive(Clone)]
pub struct WasmNodePortDescriptors {
    #[allow(dead_code)]
    node_id: NodeId,
    input_port_descriptors: Box<[PortDescriptor]>,
    output_port_descriptors: Box<[PortDescriptor]>,
}

impl WasmNodePortDescriptors {
    fn new(node_id: NodeId, input_ports: &[PortInfo], output_ports: &[PortInfo]) -> Self {
        let input_port_descriptors = input_ports
            .iter()
            .enumerate()
            .map(|(port_idx, port_info)| PortDescriptor {
                address: PortAddress::new(
                    node_id,
                    PortId::new(port_idx),
                    PortAddressDirection::Input,
                ),
                channels: port_info.channel_count,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let output_port_descriptors = output_ports
            .iter()
            .enumerate()
            .map(|(port_idx, port_info)| PortDescriptor {
                address: PortAddress::new(
                    node_id,
                    PortId::new(port_idx),
                    PortAddressDirection::Output,
                ),
                channels: port_info.channel_count,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            node_id,
            input_port_descriptors,
            output_port_descriptors,
        }
    }
}

impl DescribePorts for WasmNodePortDescriptors {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        if self.input_port_descriptors.is_empty() {
            None
        } else {
            Some(&self.input_port_descriptors)
        }
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        if self.output_port_descriptors.is_empty() {
            None
        } else {
            Some(&self.output_port_descriptors)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use crate::{
        implementations::AudioBufferMut,
        primitives::{BlockSize, CurrentTime, Sample, SampleRate},
        test_utils::TestIdGenerator,
    };

    // The WASM fixture is built by:
    // cargo build -p resonix-wasm-audio-node --target wasm32-unknown-unknown --release
    // and copied to OUT_DIR by build.rs.
    static GAIN_NODE_WASM: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/resonix_wasm_audio_node.wasm"));

    // miri cannot use the syscalls required for wasm compilation/instantiation
    #[cfg_attr(miri, ignore)]
    #[test]
    fn instantiates_from_wasm_bytes() {
        let mut id_gen = TestIdGenerator(0);
        WasmNode::new(&mut id_gen, GAIN_NODE_WASM).expect("should instantiate");
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn reports_correct_port_counts() {
        let mut id_gen = TestIdGenerator(0);
        let node = WasmNode::new(&mut id_gen, GAIN_NODE_WASM).unwrap();
        assert_eq!(node.port_info_data.input_ports.len(), 1);
        assert_eq!(node.port_info_data.output_ports.len(), 1);
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn reports_correct_channel_counts() {
        let mut id_gen = TestIdGenerator(0);
        let node = WasmNode::new(&mut id_gen, GAIN_NODE_WASM).unwrap();
        assert_eq!(node.port_info_data.input_ports[0].channel_count, 2);
        assert_eq!(node.port_info_data.output_ports[0].channel_count, 2);
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn port_descriptors_have_correct_channel_counts() {
        let mut id_gen = TestIdGenerator(0);
        let node = WasmNode::new(&mut id_gen, GAIN_NODE_WASM).unwrap();
        let desc = node.get_port_descriptors();
        assert_eq!(desc.input_port_descriptors[0].channels, 2);
        assert_eq!(desc.output_port_descriptors[0].channels, 2);
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn applies_gain_to_stereo_input() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = WasmNode::new(&mut id_gen, GAIN_NODE_WASM).unwrap();

        let block_size = 4usize;
        let channels = 2usize;

        // Input planar: ch0 = [1, 2, 3, 4], ch1 = [5, 6, 7, 8]
        let input_data: Vec<Sample> = (1..=8).map(|x: i32| Sample::from(x as f32)).collect();
        let input_buf = crate::implementations::AudioBuffer::new(&input_data, channels).unwrap();

        let mut output_data: Vec<Sample> = vec![Sample::default(); block_size * channels];
        let output_buf = AudioBufferMut::new(&mut output_data, channels).unwrap();

        let inputs: &[Option<crate::implementations::AudioBuffer<'_>>] = &[Some(input_buf)];
        let mut outputs: Vec<Option<AudioBufferMut<'_>>> = vec![Some(output_buf)];

        node.process(
            inputs,
            outputs.as_mut_slice(),
            AudioNodeCtx {
                block_size: BlockSize::from(block_size),
                current_time: CurrentTime::from(0.0),
                sample_rate: SampleRate::default(),
            },
        )
        .expect("process should not fail");

        // Gain of 0.5: ch0 = [0.5, 1.0, 1.5, 2.0], ch1 = [2.5, 3.0, 3.5, 4.0]
        let expected: Vec<Sample> = [0.5f32, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0]
            .iter()
            .map(|&x| Sample::from(x))
            .collect();

        assert_eq!(output_data, expected);
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn does_nothing_when_output_slot_disconnected() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = WasmNode::new(&mut id_gen, GAIN_NODE_WASM).unwrap();

        let block_size = 4usize;
        let channels = 2usize;
        let input_data: Vec<Sample> = (1..=8).map(|x: i32| Sample::from(x as f32)).collect();
        let input_buf = crate::implementations::AudioBuffer::new(&input_data, channels).unwrap();

        let inputs: &[Option<crate::implementations::AudioBuffer<'_>>] = &[Some(input_buf)];
        let mut outputs: Vec<Option<AudioBufferMut<'_>>> = vec![None];

        let result = node.process(
            inputs,
            outputs.as_mut_slice(),
            AudioNodeCtx {
                block_size: BlockSize::from(block_size),
                current_time: CurrentTime::from(0.0),
                sample_rate: SampleRate::default(),
            },
        );

        assert!(result.is_ok());
    }
}
