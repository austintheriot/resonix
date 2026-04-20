//! Example resonix WASM audio node: a stereo gain node.
//!
//! Host imports (namespace "resonix"):
//!   get_block_size() -> i32
//!   get_sample_rate() -> i32
//!
//! Exports:
//!   init()                              — optional; called once after instantiation
//!   get_input_count() -> i32
//!   get_output_count() -> i32
//!   get_input_channel_count(port_id: i32) -> i32
//!   get_output_channel_count(port_id: i32) -> i32
//!   get_input_buffer_ptr(port_id: i32) -> i32
//!   get_output_buffer_ptr(port_id: i32) -> i32
//!   process(block_size: i32, current_time: f64)
//!
//! Ports:
//!   input  0: audio in  (CHANNELS channels)
//!   output 0: audio out (CHANNELS channels)
//!
//! DSP: applies a fixed gain of 0.5 to each sample.

#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[link(wasm_import_module = "resonix")]
unsafe extern "C" {
    fn get_block_size() -> i32;

    #[allow(dead_code)]
    fn get_sample_rate() -> i32;
}

const CHANNELS: usize = 2;
const MAX_BLOCK_SIZE: usize = 2048;

// TODO: fix this -- we don't need to use `max block size` here -- we can just
// initialize buffers in the `init` function

// Contiguous planar staging buffers. Channel stride = block_size (set at init).
static mut INPUT_BUF: [f32; CHANNELS * MAX_BLOCK_SIZE] = [0.0; CHANNELS * MAX_BLOCK_SIZE];
static mut OUTPUT_BUF: [f32; CHANNELS * MAX_BLOCK_SIZE] = [0.0; CHANNELS * MAX_BLOCK_SIZE];

static mut BLOCK_SIZE: usize = 0;

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    unsafe {
        BLOCK_SIZE = get_block_size() as usize;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_input_count() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn get_output_count() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn get_input_channel_count(_port_id: i32) -> i32 {
    CHANNELS as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn get_output_channel_count(_port_id: i32) -> i32 {
    CHANNELS as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn get_input_buffer_ptr(_port_id: i32) -> i32 {
    core::ptr::addr_of!(INPUT_BUF) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn get_output_buffer_ptr(_port_id: i32) -> i32 {
    core::ptr::addr_of!(OUTPUT_BUF) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn process(block_size: i32, _current_time: f64) {
    let n = block_size as usize;
    for ch in 0..CHANNELS {
        let base = ch * n;
        for i in 0..n {
            unsafe {
                OUTPUT_BUF[base + i] = INPUT_BUF[base + i] * 0.5;
            }
        }
    }
}
