use super::app_state::AppState;
use yew::UseReducerHandle;

#[derive(Clone, PartialEq)]
pub struct AppContext {
    pub state_handle: UseReducerHandle<AppState>,
}

pub struct AppContextError;

impl AppContextError {
    pub const NOT_FOUND: &'static str = "AppContext was not found";
}
