use yew::{function_component, html, prelude::*};
use crate::{
    audio::{self},
    state::{app_context::{AppContext, AppContextError}, app_action::AppAction},
    components::buffer_visualizer::BufferVisualizer
};


#[function_component(AudioControls)]
pub fn audio_controls() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);

    let handle_play = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            let state_handle = state_handle.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let new_stream_handle = audio::play::play(state_handle.clone()).await;
                // save the audio stream handle so that playback continues
                // (once the handle is dropped, the stream will stop playing)
                state_handle.dispatch(AppAction::SetStreamHandle(Some(new_stream_handle)));
            })
        })
    };

    let handle_stop = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            // drop the audio stream handle to stop playback
            state_handle.dispatch(AppAction::SetStreamHandle(None));
        })
    };

    html! {
        <div class="audio-controls">
            <button
                id="play"
                class="audio-controls__play"
                onclick={handle_play}
            >
                {"Play"}
            </button>
            <button
                id="stop"
                class="audio-controls__stop"
                onclick={handle_stop}
            >
                {"Stop"}
            </button>

            <BufferVisualizer />
        </div>
    }
}
