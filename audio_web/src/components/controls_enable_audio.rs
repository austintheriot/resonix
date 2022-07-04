use crate::{
    audio::{self, play_status::PlayStatus},
    components::button::Button,
    icons::power::IconPower,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
    },
};
use yew::{function_component, html, prelude::*};

use super::button::ButtonVariant;

#[function_component(ControlsEnableAudio)]
pub fn controls_enable_audio() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let audio_is_loading = app_context.state_handle.audio_loading;
    let audio_is_initialized = app_context.state_handle.audio_initialized;
    let button_disabled = audio_is_loading;

    let handle_click = {
        let state_handle = app_context.state_handle;
        let audio_ouput_handle = app_context.audio_output_handle;
        Callback::from(move |_: MouseEvent| {
            let state_handle = state_handle.clone();
            let audio_ouput_handle =  (*audio_ouput_handle).clone();
            if button_disabled {
                
            } else if state_handle.audio_initialized {
                state_handle.dispatch(AppAction::SetAudioLoading(false));
                state_handle.dispatch(AppAction::SetAudioInitialized(false));
                state_handle.dispatch(AppAction::SetStreamHandle(None));
                state_handle.dispatch(AppAction::SetPlayStatus(PlayStatus::Pause));
            } else {
                wasm_bindgen_futures::spawn_local(async move {
                    state_handle.dispatch(AppAction::SetAudioLoading(true));
                    let new_stream_handle =
                        audio::initialize::initialize_audio(state_handle.clone(), audio_ouput_handle).await;
                    // save the audio stream handle so that playback continues
                    // (once the handle is dropped, the stream will stop playing)
                    state_handle.dispatch(AppAction::SetAudioLoading(false));
                    state_handle.dispatch(AppAction::SetAudioInitialized(true));
                    state_handle.dispatch(AppAction::SetStreamHandle(Some(new_stream_handle)));
                })
            }
        })
    };

    let default_class = "controls-enable-audio";
    let audio_loading_class = if audio_is_loading { "loading" } else { "" };
    let audio_initialized_class = if audio_is_initialized {
        "initialized"
    } else {
        ""
    };

    let aria_label = if audio_is_initialized {
        "turn off audio"
    } else {
        "turn on audio"
    };

    let button_variant = if audio_is_initialized || audio_is_loading {
        ButtonVariant::Pressed
    } else {
        ButtonVariant::Unpressed
    };

    html! {
        <Button
            aria_label={aria_label}
            disabled={button_disabled}
            class={classes!(default_class, audio_loading_class, audio_initialized_class)}
            onclick={handle_click}
            variant={button_variant}
        >
            <IconPower />
        </Button>
    }
}
