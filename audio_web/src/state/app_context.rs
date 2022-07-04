use super::app_state::AppState;
use crate::audio::audio_ouput_handle::AudioOutputHandle;
use std::rc::Rc;
use yew::{use_reducer_eq, use_ref, UseReducerHandle};

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub state_handle: UseReducerHandle<AppState>,
    pub audio_output_handle: Rc<AudioOutputHandle>,
}

pub struct AppContextError;

impl AppContextError {
    pub const NOT_FOUND: &'static str = "AppContext was not found";
}

impl Default for AppContext {
    fn default() -> Self {
        AppContext {
            state_handle: use_reducer_eq(AppState::default),
            audio_output_handle: use_ref(AudioOutputHandle::default),
        }
    }
}
