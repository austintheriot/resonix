use crate::{
    audio::{play_status::PlayStatus, play_status_action::PlayStatusAction},
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsPlayStatus)]
pub fn controls_play_status() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let play_status = app_context.state_handle.play_status_handle.get();
    let input_disabled = app_context.state_handle.get_are_audio_controls_disabled();

    let handle_change = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: InputEvent| {
            let checked = e.target_dyn_into::<HtmlInputElement>().unwrap().checked();
            let play_status = match checked {
                true => PlayStatus::PLAY,
                false => PlayStatus::PAUSE,
            };
            state_handle.dispatch(AppAction::SetPlayStatus(play_status));
        })
    };

    html! {
        <>
            <label for="controls-play-status">
                {"Play"}
            </label>
            <input
                id="controls-play-status"
                class="controls-play-status"
                type="checkbox"
                oninput={handle_change}
                checked={play_status.into()}
                disabled={input_disabled}
            />
        </>
    }
}
