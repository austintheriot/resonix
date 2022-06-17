use serde::Serialize;

pub const BUFFER_SELECTION_MIN: f32 = 0.0;
pub const BUFFER_SELECTION_MAX: f32 = 1.0;
pub const MIN_BUFFER_SELECTION_SIZE: f32 = 0.01;

/// A represents what portion of an audio buffer is currently selected,
/// ranging from 0.0 (start) to 1.0 (end).
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct BufferSelection {
    /// the start of the current selection inside the audio buffer
    pub start: f32,
    /// the end of the current selectino inside the audio buffer
    pub end: f32,
    /// reflects whether the mouse is currently being dragged inside the buffer
    pub mouse_down: bool,
}

impl Default for BufferSelection {
    fn default() -> Self {
        Self {
            start: BUFFER_SELECTION_MIN,
            end: BUFFER_SELECTION_MAX,
            mouse_down: false,
        }
    }
}

impl BufferSelection {
    pub fn set_start(mut self, start: f32) -> Self {
        let sanitized_start = start
            .min(self.end - MIN_BUFFER_SELECTION_SIZE)
            .min(BUFFER_SELECTION_MAX - MIN_BUFFER_SELECTION_SIZE)
            .max(BUFFER_SELECTION_MIN);

        self.start = sanitized_start;

        self
    }
    pub fn set_end(mut self, end: f32) -> Self {
        let sanitized_end = end
            .max(self.start + MIN_BUFFER_SELECTION_SIZE)
            .max(BUFFER_SELECTION_MIN + MIN_BUFFER_SELECTION_SIZE)
            .min(BUFFER_SELECTION_MAX);

        self.end = sanitized_end;

        self
    }
    pub fn set_mouse_down(mut self, mouse_down: bool) -> Self {
        self.mouse_down = mouse_down;
        self
    }
}
