use crate::{components::{
    controls_container::ControlsContainer,
    keyboard_listener::KeyboardListener,
}, state::app_context::AppContext};
use yew::{function_component, html, prelude::*};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <ContextProvider<AppContext> context={AppContext::default()}>
            <KeyboardListener>
                <ControlsContainer />
            </ KeyboardListener>
        </ContextProvider<AppContext>>
    }
}
