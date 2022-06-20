use super::buffer_selection::{BufferSelection, BUFFER_SELECTION_MAX, BUFFER_SELECTION_MIN};
use std::sync::{Arc, Mutex};

/// This is an `Arc` wrapper around `BufferSelection`
///
/// Wrapping `BufferSelection` in an `Arc` allows up-to-date state in BufferSelection to be
/// accesed from within the separate audio thread for up-to-date audio processing.
///
/// When the `BufferHandle` is `Clone`d on global state updates, it's clone is the
/// identical object to the original, because the `buffer_selection`'s `Arc` is simply cloned,
/// and the outer struct frame (with the `counter`) is identical.
///
/// This makes it impossible for Yew to know when the `buffer_selection`'s data has actually been
/// udpated internally, because it will always be compared to itself accross state updates.
///
/// For this reason, an internal counter is udated to "force" a difference in state
/// while keeping the underlying `buffer_selection` reference identical in memory.
///
/// This allows Yew to know that internal state has changed, while also keeping the state's
/// location in memory unchanged, so that it can be safely accessed from the audio thread.
#[derive(Clone, Debug, Default)]
pub struct BufferSelectionHandle {
    buffer_selection: Arc<Mutex<BufferSelection>>,
    counter: u32,
}

impl BufferSelectionHandle {
    /// Bumps up counter so that Yew knows interanal state has changed,
    /// even when the internal buffer_selection points to the same memory
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    pub fn set_mouse_start(&mut self, start: f32) -> &mut Self {
        self.buffer_selection.lock().unwrap().mouse_start =
            start.max(BUFFER_SELECTION_MIN).min(BUFFER_SELECTION_MAX);

        self.bump_counter();

        self
    }

    pub fn set_mouse_end(&mut self, end: f32) -> &mut Self {
        self.buffer_selection.lock().unwrap().mouse_end =
            end.max(BUFFER_SELECTION_MIN).min(BUFFER_SELECTION_MAX);

        self.bump_counter();

        self
    }

    pub fn set_mouse_down(&mut self, mouse_down: bool) -> &mut Self {
        self.buffer_selection.lock().unwrap().mouse_down = mouse_down;

        self.bump_counter();

        self
    }

    pub fn get_mouse_down(&self) -> bool {
        self.buffer_selection.lock().unwrap().mouse_down
    }

    pub fn get_mouse_start(&self) -> f32 {
        self.buffer_selection.lock().unwrap().mouse_start
    }

    pub fn get_mouse_end(&self) -> f32 {
        self.buffer_selection.lock().unwrap().mouse_end
    }

    /// Copies the existing buffer selection struct out
    pub fn get_buffer_selection(&self) -> BufferSelection {
        *self.buffer_selection.lock().unwrap()
    }

    /// Returns the mouse start / mouse end poisition in the correct order
    /// (i.e. from least to greatest / from left to right)
    ///
    /// This does not guarantee that the start and end are not the SAME number.
    pub fn get_buffer_start_and_end(&self) -> (f32, f32) {
        let buffer_selection = self.get_buffer_selection();

        if buffer_selection.mouse_start > buffer_selection.mouse_end {
            (buffer_selection.mouse_end, buffer_selection.mouse_start)
        } else {
            (buffer_selection.mouse_start, buffer_selection.mouse_end)
        }
    }

    /// Returns the mouse start position (this number is guarnateed to be <= the end position)
    pub fn get_buffer_start(&self) -> f32 {
        self.get_buffer_start_and_end().0
    }

    /// Returns the mouse end poisitino (this number is guarnateed to be >= the start position)
    pub fn get_buffer_end(&self) -> f32 {
        self.get_buffer_start_and_end().1
    }
}

impl BufferSelectionHandle {
    pub fn new(buffer_selection: Arc<Mutex<BufferSelection>>) -> Self {
        BufferSelectionHandle {
            buffer_selection,
            counter: Default::default(),
        }
    }
}

impl PartialEq for BufferSelectionHandle {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter && self.get_buffer_selection() == other.get_buffer_selection()
    }
}

impl Eq for BufferSelectionHandle {}
