use crate::{
    audio::{self},
    components::buffer_container::BufferContainer,
    components::controls_gain::ControlsGain,
    components::controls_status::ControlsStatus,
    components::controls_select_buffer::ControlsSelectBuffer,
    components::controls_density::ControlsDensity,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use yew::{function_component, html, prelude::*};

#[function_component(ControlsContainer)]
pub fn controls_container() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let enable_audio_button_disabled = app_context
        .state_handle
        .get_is_enable_audio_button_disabled();
    let audio_is_loading = app_context.state_handle.audio_loading;
    let audio_is_initialized = app_context.state_handle.audio_initialized;

    let handle_play = {
        let state_handle = app_context.state_handle.clone();
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

    html! {
        <div class="controls-container">
            {if !audio_is_initialized {
                html!{
                    <button
                        class="controls-container__enable-audio"
                        onclick={handle_play}
                        disabled={enable_audio_button_disabled}
                    >
                        {if audio_is_loading {
                        "Loading"
                        } else {
                            "Enable audio"
                        }}
                    </button>
                }
            } else {
                html!{}
            }}
            <ControlsStatus />
            <ControlsGain />
            <ControlsDensity />
            <ControlsSelectBuffer />
            <BufferContainer />
        </div>
    }
}
