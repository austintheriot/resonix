use crate::state::app_context::{AppContext, AppContextError};
use yew::prelude::*;

#[function_component(LoadingIndicator)]
pub fn loading_indicator() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let audio_loading = app_context.state_handle.audio_loading;
    let loading_class = audio_loading.then_some("loading");

    html! {
         <div class={classes!("loading-indicator", loading_class)} >
            <div class="loading-bar" />
         </div>
    }
}
