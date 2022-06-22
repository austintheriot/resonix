use serde::Serialize;
use super::buffer_selection_action::BufferSelectionAction;

/// A represents what portion of an audio buffer is currently selected,
/// ranging from 0.0 (start) to 1.0 (end).
#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct BufferSelection {
    /// The start of the current mouse selection inside the audio buffer.
    ///
    /// Because it's possile to select a buffer from right to left,
    /// the mouse start position may be GREATER than the mouse end position.
    mouse_start: f32,

    /// The end of the current mouse selection inside the audio buffer
    ///
    /// Because it's possile to select a buffer from right to left,
    /// the mouse end position may be LESS than the mouse start position.
    mouse_end: f32,

    /// reflects whether the mouse is currently being dragged inside the buffer
    mouse_down: bool,
}

impl BufferSelectionAction for BufferSelection {
    const BUFFER_SELECTION_START: f32 = 0.0;
    const BUFFER_SELECTION_END: f32 = 1.0;
    const BUFFER_SELECTION_MIN_LEN: f32 = 0.01;

    fn set_mouse_start(&mut self, start: f32) -> &mut Self {
        self.mouse_start = Self::sanitize_selection(start);

        self
    }

    fn set_mouse_end(&mut self, end: f32) -> &mut Self {
        self.mouse_end = Self::sanitize_selection(end);

        self
    }

    fn set_mouse_down(&mut self, mouse_down: bool) -> &mut Self {
        self.mouse_down = mouse_down;

        self
    }

    fn get_mouse_down(&self) -> bool {
        self.mouse_down
    }

    fn get_mouse_start(&self) -> f32 {
        self.mouse_start
    }

    fn get_mouse_end(&self) -> f32 {
        self.mouse_end
    }

    fn get_buffer_selection(&self) -> BufferSelection {
        *self
    }

    fn get_buffer_start_and_end(&self) -> (f32, f32) {
        let buffer_selection = self.get_buffer_selection();

        if buffer_selection.mouse_start > buffer_selection.mouse_end {
            (buffer_selection.mouse_end, buffer_selection.mouse_start)
        } else {
            (buffer_selection.mouse_start, buffer_selection.mouse_end)
        }
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
