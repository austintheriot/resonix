//! Example resonix WASM audio node: a stereo gain node.
//!
//! Exports required by WasmNode:
//!   resonix_input_count() -> i32
//!   resonix_output_count() -> i32
//!   resonix_get_input_channel_count(port_id: i32) -> i32
//!   resonix_get_output_channel_count(port_id: i32) -> i32
//!   resonix_get_input_channel_buffer_ptr(port_id: i32, channel_id: i32) -> i32
//!   resonix_get_output_channel_buffer_ptr(port_id: i32, channel_id: i32) -> i32
//!   resonix_process(block_size: i32, current_time: f64)
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

const CHANNELS: usize = 2;
const MAX_BLOCK_SIZE: usize = 2048;

// Staging buffers in WASM linear memory (planar: all samples for ch0, then ch1, ...)
static mut INPUT_BUF: [f32; CHANNELS * MAX_BLOCK_SIZE] = [0.0; CHANNELS * MAX_BLOCK_SIZE];
static mut OUTPUT_BUF: [f32; CHANNELS * MAX_BLOCK_SIZE] = [0.0; CHANNELS * MAX_BLOCK_SIZE];

#[unsafe(no_mangle)]
pub extern "C" fn resonix_input_count() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn resonix_output_count() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn resonix_get_input_channel_count(_port_id: i32) -> i32 {
    CHANNELS as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn resonix_get_output_channel_count(_port_id: i32) -> i32 {
    CHANNELS as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn resonix_get_input_channel_buffer_ptr(_port_id: i32, channel_id: i32) -> i32 {
    let base = core::ptr::addr_of!(INPUT_BUF) as usize;
    (base + channel_id as usize * MAX_BLOCK_SIZE * 4) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn resonix_get_output_channel_buffer_ptr(_port_id: i32, channel_id: i32) -> i32 {
    let base = core::ptr::addr_of!(OUTPUT_BUF) as usize;
    (base + channel_id as usize * MAX_BLOCK_SIZE * 4) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn resonix_process(block_size: i32, _current_time: f64) {
    let n = block_size as usize;
    for ch in 0..CHANNELS {
        let base = ch * MAX_BLOCK_SIZE;
        for i in 0..n {
            unsafe {
                OUTPUT_BUF[base + i] = INPUT_BUF[base + i] * 0.5;
            }
        }
    }
}
