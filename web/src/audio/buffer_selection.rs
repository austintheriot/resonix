use serde::Serialize;

pub const BUFFER_SELECTION_MIN: f32 = 0.0;
pub const BUFFER_SELECTION_MAX: f32 = 1.0;
pub const MIN_BUFFER_SELECTION_SIZE: f32 = 0.01;

/// A represents what portion of an audio buffer is currently selected,
/// ranging from 0.0 (start) to 1.0 (end).
#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct BufferSelection {
    /// The start of the current mouse selection inside the audio buffer.
    /// 
    /// Because it's possile to select a buffer from right to left,
    /// the mouse start position may be GREATER than the mouse end position.
    pub mouse_start: f32,
    /// The end of the current mouse selection inside the audio buffer
    /// 
    /// Because it's possile to select a buffer from right to left,
    /// the mouse end position may be LESS than the mouse start position.
    pub mouse_end: f32,
    /// reflects whether the mouse is currently being dragged inside the buffer
    pub mouse_down: bool,
}

impl Default for BufferSelection {
    fn default() -> Self {
        Self {
            mouse_start: BUFFER_SELECTION_MIN,
            mouse_end: BUFFER_SELECTION_MAX,
            mouse_down: false,
        }
    }
}
