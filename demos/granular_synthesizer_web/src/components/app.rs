use crate::{
    components::{controls_container::ControlsContainer, keyboard_listener::KeyboardListener},
    state::{app_context::AppContext, app_state::AppState},
};
use yew::{function_component, html, prelude::*};

#[function_component(App)]
pub fn app() -> Html {
    let use_reducer_handle = use_reducer_eq(AppState::default);
    let context = AppContext::from(use_reducer_handle);
    html! {
        <ContextProvider<AppContext> {context}>
            <KeyboardListener>
                <ControlsContainer />
            </KeyboardListener>
        </ContextProvider<AppContext>>
    }
}
