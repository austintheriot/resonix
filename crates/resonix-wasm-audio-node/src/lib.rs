#![no_std]

extern crate alloc;

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec::Vec};
use dlmalloc::GlobalDlmalloc;
use resonix_core::{
    implementations::OwnedAudioBuffer,
    primitives::{AudioNodeCtx, BlockSize, CurrentTime, PortDescriptor, Sample, SampleRate},
    traits::{AudioBuffer, AudioNode, DescribePorts, GetPortDescriptors},
};
use spin::{Mutex, Once};

// TODO: remove mention of `init()` from spec--it isn't needed
// TODO: add __ to function exports to prevent name collisions

#[global_allocator]
static ALLOC: GlobalDlmalloc = GlobalDlmalloc;

mod erased_audio_node;

use crate::erased_audio_node::ErasedAudioNode;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" {
    #[doc(hidden)]
    pub fn _get_block_size() -> i32;

    #[doc(hidden)]
    pub fn _get_sample_rate() -> i32;
}

pub fn get_block_size() -> BlockSize {
    let block_size = unsafe { _get_block_size() };
    BlockSize::from(block_size)
}

pub fn get_sample_rate() -> SampleRate {
    let sample_rate = unsafe { _get_sample_rate() };
    SampleRate::from(sample_rate)
}

// SAFETY: our graph execution & wasm execution
// is currently always single-threaded
unsafe impl Sync for InstanceStorage {}
unsafe impl Send for InstanceStorage {}

struct AudioBuffers {
    inputs: Box<[Option<OwnedAudioBuffer>]>,
    outputs: Box<[Option<OwnedAudioBuffer>]>,
}

fn allocate_audio_buffers_for_ports(
    block_size: usize,
    port_descriptors: &[PortDescriptor],
) -> Box<[Option<OwnedAudioBuffer>]> {
    let storage_capacity = port_descriptors
        .iter()
        .map(|port_descriptor| **port_descriptor.address.port_id())
        .max()
        .unwrap_or(0);
    let mut buffers: Vec<Option<OwnedAudioBuffer>> = Vec::with_capacity(storage_capacity);
    buffers.resize_with(storage_capacity, || None);

    for port_descriptor in port_descriptors {
        let total_samples = port_descriptor.channels * block_size;

        let mut buffer: Vec<Sample> = Vec::with_capacity(total_samples);
        buffer.resize(total_samples, Sample::default());

        buffers[**port_descriptor.address.port_id()] = Some(OwnedAudioBuffer::from_sample_buffer(
            buffer.into_boxed_slice(),
            port_descriptor.channels,
        ));
    }

    buffers.into_boxed_slice()
}

fn create_storage(
    block_size: usize,
    input_port_descriptors: &[PortDescriptor],
    output_port_descriptors: &[PortDescriptor],
) -> AudioBuffers {
    let inputs = allocate_audio_buffers_for_ports(block_size, input_port_descriptors);
    let outputs = allocate_audio_buffers_for_ports(block_size, output_port_descriptors);

    AudioBuffers { inputs, outputs }
}

struct InstanceStorage {
    audio_node: Box<dyn ErasedAudioNode>,
    port_descriptors: Box<dyn DescribePorts>,
    // once allocated, must never be re-allocated:
    // host relies on pointers into buffer memory to
    // perform I/O for audio buffers
    audio_buffers: AudioBuffers,
}

// TODO: do we need Once & Mutex ? Ideally, we would just do
// a thread_local storage
static INSTANCE: Once<Mutex<InstanceStorage>> = Once::new();

#[doc(hidden)]
pub fn register_audio_node<
    P: DescribePorts + 'static,
    A: AudioNode + GetPortDescriptors<P> + 'static,
>(
    audio_node: A,
) {
    let port_descriptors = audio_node.get_port_descriptors();
    let block_size = get_block_size();
    let audio_buffers = create_storage(
        *block_size,
        port_descriptors.input_ports().unwrap_or(&[]),
        port_descriptors.output_ports().unwrap_or(&[]),
    );

    if INSTANCE.get().is_some() {
        return;
    }

    INSTANCE.call_once(move || {
        Mutex::new(InstanceStorage {
            audio_node: Box::new(audio_node),
            port_descriptors: Box::new(port_descriptors),
            audio_buffers,
        })
    });
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn get_input_count() -> i32 {
    let instance = INSTANCE.get().expect("Instance must be initialized");
    let instance = instance.lock();
    let InstanceStorage {
        port_descriptors, ..
    } = &*instance;

    let Some(input_ports) = port_descriptors.input_ports() else {
        return 0;
    };

    input_ports.len() as i32
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn get_output_count() -> i32 {
    let instance = INSTANCE.get().expect("Instance must be initialized");
    let instance = instance.lock();
    let InstanceStorage {
        port_descriptors, ..
    } = &*instance;

    let Some(output_ports) = port_descriptors.output_ports() else {
        return 0;
    };

    output_ports.len() as i32
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn get_input_channel_count(port_id: i32) -> i32 {
    let instance = INSTANCE.get().expect("Instance must be initialized");
    let instance = instance.lock();
    let InstanceStorage {
        port_descriptors, ..
    } = &*instance;

    let Some(input_port_descriptors) = port_descriptors.input_ports() else {
        return 0;
    };

    let Some(input_port_descriptor) = input_port_descriptors.get(port_id as usize) else {
        return 0;
    };

    input_port_descriptor.channels as i32
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn get_output_channel_count(port_id: i32) -> i32 {
    let instance = INSTANCE.get().expect("Instance must be initialized");
    let instance = instance.lock();
    let InstanceStorage {
        port_descriptors, ..
    } = &*instance;

    let Some(output_port_descriptors) = port_descriptors.output_ports() else {
        return 0;
    };

    let Some(output_port_descriptor) = output_port_descriptors.get(port_id as usize) else {
        return 0;
    };

    output_port_descriptor.channels as i32
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn get_input_buffer_ptr(port_id: i32) -> i32 {
    let instance = INSTANCE.get().expect("Instance must be initialized");
    let instance = instance.lock();
    let InstanceStorage { audio_buffers, .. } = &*instance;

    audio_buffers
        .inputs
        .get(port_id as usize)
        .unwrap()
        .as_ref()
        .unwrap()
        .as_slice()
        .as_ptr() as i32
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn get_output_buffer_ptr(port_id: i32) -> i32 {
    let instance = INSTANCE.get().expect("Instance must be initialized");
    let instance = instance.lock();
    let InstanceStorage { audio_buffers, .. } = &*instance;

    audio_buffers
        .outputs
        .get(port_id as usize)
        .unwrap()
        .as_ref()
        .unwrap()
        .as_slice()
        .as_ptr() as i32
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn process(current_time: f64) {
    let instance = INSTANCE.get().expect("Instance must be initialized");
    let mut instance = instance.lock();
    let InstanceStorage {
        audio_node,
        audio_buffers,
        ..
    } = &mut *instance;

    let block_size = get_block_size();
    let sample_rate = get_sample_rate();
    let ctx = AudioNodeCtx::builder()
        .block_size(BlockSize::from(block_size))
        .current_time(CurrentTime::from(current_time))
        .sample_rate(SampleRate::from(sample_rate))
        .build();

    audio_node
        .process(&audio_buffers.inputs, &mut audio_buffers.outputs, ctx)
        .expect("audio node `process` threw error internally");
}

#[doc(hidden)]
#[macro_export]
macro_rules! export_wasm_api {
    () => {
        pub use $crate::get_input_buffer_ptr;
        pub use $crate::get_input_channel_count;
        pub use $crate::get_input_count;
        pub use $crate::get_output_buffer_ptr;
        pub use $crate::get_output_channel_count;
        pub use $crate::get_output_count;
        pub use $crate::process;
    };
}

pub use resonix_wasm_audio_node_macros::*;
