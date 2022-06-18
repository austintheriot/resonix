use super::buffer_handle::BufferHandle;
use crate::audio::buffer::Buffer;
use crate::audio::stream_handle::StreamHandle;
use crate::state::app_action::AppAction;
use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AppState {
    /// the currently loaded audio buffer
    pub buffer: Buffer,
    /// a handle to the audio context stream (keeps audio playing & stops audio when dropped)
    pub stream_handle: Option<StreamHandle>,
    /// represents what portion of the audio buffer is currently selected
    pub buffer_handle: BufferHandle,
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();
        {
            let action = action.clone();
            match action {
                AppAction::SetBuffer(buffer) => {
                    next_state.buffer = Buffer::new(buffer);
                }
                AppAction::SetStreamHandle(stream_handle) => {
                    next_state.stream_handle = stream_handle;
                }
                AppAction::SetBufferSelectionStart(start) => {
                    next_state
                        .buffer_handle
                        .buffer_selection
                        .lock()
                        .unwrap()
                        .set_start(start);

                    // assume that the date changed inside the buffer selection
                    next_state.buffer_handle = next_state.buffer_handle.clone_with_new_id();
                }
                AppAction::SetBufferSelectionEnd(end) => {
                    next_state
                        .buffer_handle
                        .buffer_selection
                        .lock()
                        .unwrap()
                        .set_end(end);

                    // assume that the date changed inside the buffer selection
                    next_state.buffer_handle = next_state.buffer_handle.clone_with_new_id();
                }
                AppAction::SetBufferSelectionMouseDown(mouse_down) => {
                    next_state
                        .buffer_handle
                        .buffer_selection
                        .lock()
                        .unwrap()
                        .set_mouse_down(mouse_down);

                    // assume that the date changed inside the buffer selection
                    next_state.buffer_handle = next_state.buffer_handle.clone_with_new_id();
                }
            }
        }

        Rc::new(next_state)
    }
}
