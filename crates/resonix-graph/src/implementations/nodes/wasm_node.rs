use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Deref;

use thiserror::Error;
use wasmer::{CompileError, Instance, InstantiationError, Module, Store, imports};

use crate::{
    errors::AudioNodeRunError,
    primitives::{AudioNodeCtx, Id, NodeId, PortDescriptor, Priority},
    traits::{
        AudioBuffer, AudioBufferMut, AudioNode, DescribePorts, GenerateId, GetNodeId,
        GetPortDescriptors, GetPriority,
    },
};

pub struct WasmNode {
    node_id: NodeId,
    instance: Instance,
    port_descriptors: WasmNodePortDescriptors,
}

#[derive(Error, Debug)]
pub enum WasmNodeCreationError {
    #[error("wasm node encountered error while compiling: {0:?}")]
    CompileError(#[from] CompileError),
    #[error("wasm node encountered error while instantiating {0:?}")]
    InstantiationError(#[from] InstantiationError),
}

impl WasmNode {
    pub fn new<G: GenerateId>(
        id_generator: &mut G,
        bytes: &[u8],
    ) -> Result<Self, WasmNodeCreationError> {
        let mut store = Store::default();
        let module = Module::new(&store, bytes)?;
        // The module doesn't import anything, so we create an empty import object.
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;

        let node_id = NodeId::from(id_generator.generate_id());
        let port_descriptors = WasmNodePortDescriptors::new(node_id);

        Ok(Self {
            node_id,
            instance,
            port_descriptors,
        })
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

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for WasmNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for WasmNode {
    fn process<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        _inputs: &[Option<A>],
        _outputs: &mut [Option<M>],
        _ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError> {
        todo!();
    }
}

impl Deref for WasmNode {
    type Target = WasmNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestIdGenerator;

    // miri cannot use the syscalls required for wasm compilation/instantiation
    #[cfg_attr(not(miri), test)]
    fn compiles_wasm_module_without_throwing() {
        let mut test_id_generator = TestIdGenerator(0);
        // TODO: replace with real, compiled module
        let module_wat = r#"(module
      (type $t0 (func (param i32) (result i32)))
      (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        local.get $p0
        i32.const 1
        i32.add))"#;

        WasmNode::new(&mut test_id_generator, module_wat.as_bytes()).unwrap();
    }
}

#[derive(Clone)]
pub struct WasmNodePortDescriptors {
    node_id: NodeId,
    input_port_descriptors: Box<[PortDescriptor]>,
    output_port_descriptors: Box<[PortDescriptor]>,
}

impl WasmNodePortDescriptors {
    pub fn new(node_id: NodeId) -> Self {
        // TODO: actually implement
        let input_port_descriptors = Vec::new().into_boxed_slice();
        let output_port_descriptors = Vec::new().into_boxed_slice();

        Self {
            node_id,
            input_port_descriptors,
            output_port_descriptors,
        }
    }
}

impl DescribePorts for WasmNodePortDescriptors {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        todo!()
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        todo!()
    }
}
