use super::{buffer_selection::BufferSelection};
use crate::audio::buffer::Buffer;
use crate::audio::stream_handle::StreamHandle;
use crate::state::app_action::AppAction;
use serde::{Serialize};
use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
pub struct AppState {
    /// the currently loaded audio buffer
    pub buffer: Buffer,
    /// a handle to the audio context stream (keeps audio playing & stops audio when dropped)
    pub stream_handle: Option<StreamHandle>,
    /// represents what portion of the audio buffer is currently selected
    pub buffer_selection: BufferSelection,
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
                    next_state.buffer_selection.set_start(start);
                }
                AppAction::SetBufferSelectionEnd(end) => {
                    next_state.buffer_selection.set_end(end);
                }
                AppAction::SetBufferSelectionMouseDown(mouse_down) => {
                    next_state.buffer_selection.set_mouse_down(mouse_down);
                }
            }
        }

        Rc::new(next_state)
    }
}
