use serde::Serialize;

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

impl BufferSelection {
    pub const BUFFER_SELECTION_START: f32 = 0.0;
    pub const BUFFER_SELECTION_END: f32 = 1.0;
    pub const BUFFER_SELECTION_MIN_LEN: f32 = 0.01;

    pub fn sanitize_selection(start_or_end: f32) -> f32 {
        start_or_end
            .max(BufferSelection::BUFFER_SELECTION_START)
            .min(BufferSelection::BUFFER_SELECTION_END)
    }
}

impl Default for BufferSelection {
    fn default() -> Self {
        Self {
            mouse_start: BufferSelection::BUFFER_SELECTION_START,
            mouse_end: BufferSelection::BUFFER_SELECTION_END,
            mouse_down: false,
        }
    }
}
