use std::future::Future;

use crate::AudioContextOptions;

/// Underlying storage for the audio processes, audio graph, etc.
pub trait AudioContext {
    fn new_with_options(options: impl AudioContextOptions) -> impl Future<Output = Self>;

    fn new() -> impl Future<Output = Self>;
}
