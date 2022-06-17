use serde::Serialize;

pub const BUFFER_SELECTION_MIN: f32 = 0.0;
pub const BUFFER_SELECTION_MAX: f32 = 1.0;
pub const MIN_BUFFER_SELECTION_SIZE: f32 = 0.01;

/// A represents what portion of an audio buffer is currently selected,
/// ranging from 0.0 (start) to 1.0 (end).
#[derive(Clone, PartialEq, Debug, Serialize)]
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

impl BufferSelection {
    pub fn set_start(mut self, start: f32) -> Self {
        self.mouse_start = start.max(BUFFER_SELECTION_MIN).min(BUFFER_SELECTION_MAX);
        self
    }
    pub fn set_end(mut self, end: f32) -> Self {
        self.mouse_end = end.max(BUFFER_SELECTION_MIN).min(BUFFER_SELECTION_MAX);
        self
    }
    pub fn set_mouse_down(mut self, mouse_down: bool) -> Self {
        self.mouse_down = mouse_down;
        self
    }

    /// Returns the mouse start / mouse end poisition in the correct order
    /// (i.e. from least to greatest / from left to right)
    /// 
    /// This does not guarantee that the start and end are not the SAME number.
    pub fn get_buffer_start_and_end(&self) -> (f32, f32) {
        if self.mouse_start > self.mouse_end {
            (self.mouse_end, self.mouse_start)
        } else {
            (self.mouse_start, self.mouse_end)
        }
    }

    pub fn get_buffer_start(&self) -> f32 {
        self.get_buffer_start_and_end().0
    }

    pub fn get_buffer_end(&self) -> f32 {
        self.get_buffer_start_and_end().1
    }
}
