use super::app_state::AppState;
use yew::UseReducerHandle;

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub state_handle: UseReducerHandle<AppState>,
}

impl AppContext {
    pub fn new(state_handle: UseReducerHandle<AppState>) -> Self {
        Self { state_handle }
    }
}

impl From<UseReducerHandle<AppState>> for AppContext {
    fn from(value: UseReducerHandle<AppState>) -> Self {
        Self::new(value)
    }
}

pub struct AppContextError;

impl AppContextError {
    pub const NOT_FOUND: &'static str = "AppContext was not found";
}
