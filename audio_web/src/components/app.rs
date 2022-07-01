use crate::{components::controls_container::ControlsContainer, state::app_context::AppContext};
use yew::{function_component, html, prelude::*};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <ContextProvider<AppContext> context={AppContext::default()}>
            <ControlsContainer />
        </ContextProvider<AppContext>>
    }
}
