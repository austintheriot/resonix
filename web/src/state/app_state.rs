use std::{rc::Rc};
use yew::Reducible;
use crate::audio::buffer::Buffer;
use crate::audio::stream_handle::StreamHandle;
use crate::state::app_action::AppAction;

#[derive(Clone, Debug, PartialEq, Default )]
pub struct AppState {
    pub buffer: Buffer,
    pub stream_handle: Option<StreamHandle>
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
            }
        }

        // log_state_update(action, (*self).clone(), next_state.clone());
        Rc::new(next_state)
    }
}