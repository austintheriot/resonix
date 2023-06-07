use nohash_hasher::IsEnabled;
use core::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ConcatenativeSynthesizerGrain {
    pub buffer: Arc<Vec<f32>>,
    pub start_frame: usize,
    pub end_frame: usize,
    pub current_frame: usize,
    /// the number of frames between `start_frame` and `end_frame` in samples
    pub len: usize,
    /// allows O(1) look-ups when finding grains that are finished
    pub uid: u32,
    /// Whether the grain exceeds the current buffer selection
    pub exceeds_buffer_selection: bool,
    /// `true` when the Grain has played through the full range of its selection
    pub is_finished: bool,
    pub average_pitch: f32,
}

impl IsEnabled for ConcatenativeSynthesizerGrain {}

/// Grain ids are guaranteed to be unique, so it is sufficient to hash based off of the uid
impl Hash for ConcatenativeSynthesizerGrain {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl ConcatenativeSynthesizerGrain {
    pub fn next_sample(&mut self) -> Option<usize> {
        if self.is_finished {
            return None;
        }

        // return the frame that was valid before incrementing the count
        let frame_to_return = self.current_frame;

        self.current_frame += 1;
        if self.current_frame == self.end_frame {
            self.is_finished = true;
        }

        Some(frame_to_return)
    }
}