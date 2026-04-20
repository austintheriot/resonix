mod erased_audio_node;

use erased_audio_node::*;

thread_local! {
    static AUDIO_NODE: RefCell<dyn AudioNode> = RefCell::new(vec![1.0, 2.0]);
}


unsafe extern "C" {
    pub fn get_block_size() -> i32;

    pub fn get_sample_rate() -> i32;
}

#[unsafe(no_mangle)]
pub extern "C" fn init() { todo!(); }

#[unsafe(no_mangle)]
pub extern "C" fn get_input_count() -> i32 { todo!(); }

#[unsafe(no_mangle)]
pub extern "C" fn get_output_count() -> i32 { todo!(); }

#[unsafe(no_mangle)]
pub extern "C" fn get_input_channel_count(_port_id: i32) -> i32 { todo!(); }

#[unsafe(no_mangle)]
pub extern "C" fn get_output_channel_count(_port_id: i32) -> i32 { todo!(); }

#[unsafe(no_mangle)]
pub extern "C" fn get_input_buffer_ptr(_port_id: i32) -> i32 { todo!(); }

#[unsafe(no_mangle)]
pub extern "C" fn get_output_buffer_ptr(_port_id: i32) -> i32 { todo!(); }

#[unsafe(no_mangle)]
pub extern "C" fn process(_block_size: i32, _current_time: f64) { todo!(); }
