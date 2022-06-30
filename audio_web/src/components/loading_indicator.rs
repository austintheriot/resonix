use yew::prelude::*;
use crate::state::app_context::{AppContext, AppContextError};

#[derive(Properties, PartialEq)]
pub struct LoadingIndicatorProps {
    #[prop_or_default]
    pub example: u32,
    #[prop_or(1)]
    pub example_1: u32,
    #[prop_or_default]
    pub example_2: u32,
}

#[function_component(LoadingIndicator)]
pub fn loading_indicator(props: &LoadingIndicatorProps) -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let audio_loading = app_context.state_handle.audio_loading;
    let loading_class = audio_loading.then(|| "loading");

    html! {
         <div class={classes!("loading-indicator", loading_class)} >
            <div class="loading-bar" />
         </div>
    }
}
