use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Clock {
    /// a u32 that is incremented 48,000 times a second will overflow after a day
    pub now: Arc<Mutex<u32>>,
}