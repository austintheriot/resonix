use crate::{
    components::button::Button,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use yew::{function_component, html, prelude::*};

#[function_component(ControlsDownloadAudio)]
pub fn controls_download_audio() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let button_disabled = app_context.state_handle.get_are_audio_controls_disabled();

    let handle_click = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }
            state_handle.dispatch(AppAction::DownloadAudio);
        })
    };

    html! {
        <Button
            aria_label="download recorded audio"
            class="controls-download-audio"
            onclick={handle_click}
            disabled={button_disabled}
        >
            {"D"}
        </Button>
    }
}
