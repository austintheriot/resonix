use yew::{function_component, html, prelude::*};
use crate::{
    audio::{self},
    state::{app_context::{AppContext, AppContextError}, app_action::AppAction, app_selector::AppSelector},
    components::buffer_container::BufferContainer,
    components::controls_gain::ControlsGain,
    components::controls_status::ControlsStatus,
};


#[function_component(ControlsContainer)]
pub fn controls_container() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let enable_audio_button_disabled = app_context.state_handle.get_is_enable_audio_button_disabled();

    let handle_play = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            let state_handle = state_handle.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let new_stream_handle = audio::initialize_audio::initialize_audio(state_handle.clone()).await;
                // save the audio stream handle so that playback continues
                // (once the handle is dropped, the stream will stop playing)
                state_handle.dispatch(AppAction::SetAudioInitialized(true));
                state_handle.dispatch(AppAction::SetStreamHandle(Some(new_stream_handle)));
            })
        })
    };

    html! {
        <div class="controls-container">
            <button
                class="controls-container__enable-audio"
                onclick={handle_play}
                disabled={enable_audio_button_disabled}
            >
                {"Enable audio"}
            </button>
            <ControlsStatus />
            <ControlsGain />
            <BufferContainer />
        </div>
    }
}
