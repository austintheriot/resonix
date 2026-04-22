#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use dlmalloc::GlobalDlmalloc;
use resonix_core::traits::{AudioNode, DescribePorts, GetPortDescriptors};

#[global_allocator]
static ALLOC: GlobalDlmalloc = GlobalDlmalloc;

mod erased_audio_node;

use core::{
    cell::{Ref, RefCell},
    panic::PanicInfo,
};

use crate::erased_audio_node::ErasedAudioNode;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" {
    pub fn get_block_size() -> i32;
    pub fn get_sample_rate() -> i32;
}

// resonix is currently single-threaded only, so this is fine
unsafe impl Sync for AudioNodeStorage {}

struct AudioNodeStorage(RefCell<Option<Box<dyn ErasedAudioNode>>>);

impl AudioNodeStorage {
    fn get(&self) -> Option<&dyn ErasedAudioNode> {
        self.0.as_ref().map(|v| &**v)
    }

    fn get_mut(&mut self) -> Option<&mut (dyn ErasedAudioNode + 'static)> {
        self.0.as_deref_mut()
    }

    fn set<A: AudioNode + 'static>(&mut self, audio_node: A) {
        self.0 = RefCell::new(Some(Box::new(audio_node) as Box<dyn ErasedAudioNode>))
    }
}

static AUDIO_NODE_STORAGE: AudioNodeStorage = AudioNodeStorage(RefCell::new(None));

pub fn register_audio_node<P: DescribePorts, A: AudioNode + GetPortDescriptors<P> + 'static>(
    audio_node: A,
) {
    let descriptors = audio_node.get_port_descriptors();

    if AUDIO_NODE_STORAGE.borrow().get().is_some() {
        return;
    }

    let mut audio_node_storage = AUDIO_NODE_STORAGE.borrow_mut();
    audio_node_storage.set(audio_node);
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn get_input_count() -> i32 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn get_output_count() -> i32 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn get_input_channel_count(_port_id: i32) -> i32 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn get_output_channel_count(_port_id: i32) -> i32 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn get_input_buffer_ptr(_port_id: i32) -> i32 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn get_output_buffer_ptr(_port_id: i32) -> i32 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn process(_block_size: i32, _current_time: f64) {
    todo!();
}
