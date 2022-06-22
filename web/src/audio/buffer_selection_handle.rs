use super::{
    buffer_selection::BufferSelection, buffer_selection_action::BufferSelectionAction,
    bump_counter::BumpCounter,
};
use std::sync::{Arc, Mutex};

/// This is an `Arc` wrapper around `BufferSelection`
///
/// Wrapping `BufferSelection` in an `Arc` allows up-to-date state in BufferSelection to be
/// accesed from within the separate audio thread for up-to-date audio processing.
///
/// When the `BufferHandle` is `Clone`d on global state updates, its clone is the
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

impl BumpCounter for BufferSelectionHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl BufferSelectionAction for BufferSelectionHandle {
    const BUFFER_SELECTION_END: f32 = BufferSelection::BUFFER_SELECTION_END;
    const BUFFER_SELECTION_START: f32 = BufferSelection::BUFFER_SELECTION_START;
    const BUFFER_SELECTION_MIN_LEN: f32 = BufferSelection::BUFFER_SELECTION_MIN_LEN;

    fn set_mouse_start(&mut self, start: f32) -> &mut Self {
        self.buffer_selection.lock().unwrap().set_mouse_start(start);
        self.bump_counter();

        self
    }

    fn set_mouse_end(&mut self, end: f32) -> &mut Self {
        self.buffer_selection.lock().unwrap().set_mouse_end(end);
        self.bump_counter();

        self
    }

    fn set_mouse_down(&mut self, mouse_down: bool) -> &mut Self {
        self.buffer_selection.lock().unwrap().set_mouse_down(mouse_down);
        self.bump_counter();

        self
    }

    fn get_mouse_down(&self) -> bool {
        self.buffer_selection.lock().unwrap().get_mouse_down()
    }

    fn get_mouse_start(&self) -> f32 {
        self.buffer_selection.lock().unwrap().get_buffer_start()
    }

    fn get_mouse_end(&self) -> f32 {
        self.buffer_selection.lock().unwrap().get_mouse_end()
    }

    /// Copies the existing buffer selection struct out
    fn get_buffer_selection(&self) -> BufferSelection {
        self.buffer_selection.lock().unwrap().get_buffer_selection()
    }

    fn get_buffer_start_and_end(&self) -> (f32, f32) {
        self.buffer_selection.lock().unwrap().get_buffer_start_and_end()
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
