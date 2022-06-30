use crate::{
    components::controls_container::ControlsContainer,
    state::{app_context::AppContext, app_state::AppState},
};
use yew::{function_component, html, prelude::*};

#[function_component(App)]
pub fn app() -> Html {
    // initialize app state
    let app_reducer_handle = use_reducer_eq(AppState::default);

    // enables updating global state in child components
    let app_context = AppContext {
        state_handle: app_reducer_handle,
    };

    html! {
        <ContextProvider<AppContext> context={app_context}>
           <ControlsContainer />
        </ContextProvider<AppContext>>
    }
}
