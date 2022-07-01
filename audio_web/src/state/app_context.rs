use super::app_state::AppState;
use yew::{UseReducerHandle, use_reducer_eq};

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub state_handle: UseReducerHandle<AppState>,
}

pub struct AppContextError;

impl AppContextError {
    pub const NOT_FOUND: &'static str = "AppContext was not found";
}

impl Default for AppContext {
    fn default() -> Self {
        AppContext {
            state_handle: use_reducer_eq(AppState::default),
        }
    }
}
