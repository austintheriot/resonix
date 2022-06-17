use crate::{
    components::audio_controls::AudioControls,
    state::{app_context::AppContext, app_state::AppState},
};
use yew::{function_component, html, prelude::*, use_reducer};

#[function_component(App)]
pub fn app() -> Html {
    // initialize app state
    let app_reducer_handle = use_reducer(AppState::default);

    // enables updating global state in child components
    let app_context = AppContext {
        state_handle: app_reducer_handle.clone(),
    };

    html! {
        <ContextProvider<AppContext> context={app_context}>
           <AudioControls />
        </ContextProvider<AppContext>>
    }
}
