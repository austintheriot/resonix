use crate::{
    audio::{self},
    icons::power::IconPower,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use yew::{function_component, html, prelude::*};

#[function_component(ControlsEnableAudio)]
pub fn controls_enable_audio() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let enable_audio_button_disabled = app_context
        .state_handle
        .get_is_enable_audio_button_disabled();
    let audio_is_loading = app_context.state_handle.audio_loading;
    let audio_is_initialized = app_context.state_handle.audio_initialized;

    let handle_click = {
        let state_handle = app_context.state_handle;
        Callback::from(move |_: MouseEvent| {
            let state_handle = state_handle.clone();
            wasm_bindgen_futures::spawn_local(async move {
                state_handle.dispatch(AppAction::SetAudioLoading(true));
                let new_stream_handle =
                    audio::initialize::initialize_audio(state_handle.clone()).await;
                // save the audio stream handle so that playback continues
                // (once the handle is dropped, the stream will stop playing)
                state_handle.dispatch(AppAction::SetAudioLoading(false));
                state_handle.dispatch(AppAction::SetAudioInitialized(true));
                state_handle.dispatch(AppAction::SetStreamHandle(Some(new_stream_handle)));
            })
        })
    };

    let default_class = "controls-enable-audio";
    let audio_loading_class = if audio_is_loading { "loading" } else { "" };
    let audio_initialized_class = if audio_is_initialized {
        "initialized"
    } else {
        ""
    };

    html! {
        <button
            class={classes!(default_class, audio_loading_class, audio_initialized_class)}
            onclick={handle_click}
            disabled={enable_audio_button_disabled}
        >
            <IconPower />
        </button>
    }
}
