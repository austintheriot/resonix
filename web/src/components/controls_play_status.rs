use crate::{
    audio::{play_status::PlayStatus, play_status_action::PlayStatusAction},
    icons::{pause::IconPause, play::IconPlay},
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use yew::{function_component, html, prelude::*};

#[function_component(ControlsPlayStatus)]
pub fn controls_play_status() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let play_status = app_context.state_handle.play_status_handle.get();
    let button_disabled = app_context.state_handle.get_are_audio_controls_disabled();

    let handle_play = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }
            state_handle.dispatch(AppAction::SetPlayStatus(PlayStatus::PLAY));
        })
    };

    let handle_pause = {
        let state_handle = app_context.state_handle;
        Callback::from(move |_: MouseEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }
            state_handle.dispatch(AppAction::SetPlayStatus(PlayStatus::PAUSE));
        })
    };

    match play_status {
        PlayStatus::PLAY => html! {
            <button
                class="controls-play-status pause"
                type="button"
                onclick={handle_pause}
                disabled={button_disabled}
            >
                <IconPause />
            </button>
        },
        PlayStatus::PAUSE => html! {
            <button
                class="controls-play-status play"
                type="button"
                onclick={handle_play}
                disabled={button_disabled}
            >
                <IconPlay />
            </button>
        },
    }
}
