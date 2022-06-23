use std::sync::Arc;

use crate::{
    audio::decode,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use gloo_net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;
use yew::{function_component, html, prelude::*};

// this list of audio files is generated statically at build time
// so that the list of default audio files always matches the actual list
// of files available from the `audio` directory
include!(concat!(env!("OUT_DIR"), "/audio_files.rs"));

/// This is the audio file that is loaded by default at initialization time
pub const DEFAULT_AUDIO_FILE: &'static str = AUDIO_FILES[3];

#[function_component(ControlsSelectBuffer)]
pub fn controls_select_buffer() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let select_ref = use_node_ref();
    let select_element_disabled = app_context.state_handle.get_are_audio_controls_disabled();

    let handle_change = {
        let state_handle = app_context.state_handle.clone();
        let select_ref = select_ref.clone();
        Callback::from(move |_: Event| {
            let state_handle = state_handle.clone();
            let select_ref = select_ref.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let select_element = select_ref
                    .get()
                    .unwrap()
                    .dyn_into::<HtmlSelectElement>()
                    .unwrap();
                let selected_index = select_element.selected_index();
                let request_url = format!("./{}", AUDIO_FILES[selected_index as usize]);

                // audio files are copied into static directory for web (same directory asthe source wasm file)
                let mp3_file_bytes = Request::get(&request_url)
                    .send()
                    .await
                    .unwrap()
                    .binary()
                    .await
                    .unwrap();

                // @todo: initialize a single audio_context at the top level of the app
                let audio_context = web_sys::AudioContext::new()
                    .expect("Browser should have AudioContext implemented");
                let audio_buffer = decode::decode_bytes(&audio_context, &mp3_file_bytes).await;
                let buffer_data = Arc::new(audio_buffer.get_channel_data(0).unwrap());
                state_handle.dispatch(AppAction::SetBuffer(buffer_data));
            })
        })
    };

    html! {
        <>
            <label for="controls-select-buffer">
                {"Select File"}
            </label>
            <select
                id="controls-select-buffer"
                class="controls-select-buffer"
                onchange={handle_change}
                ref={select_ref}
                disabled={select_element_disabled}
            >
                {AUDIO_FILES.iter().map(|file_name| {
                    html!{
                        <option selected={file_name == &DEFAULT_AUDIO_FILE}>
                            {file_name}
                        </option>
                    }
                }).collect::<Html>()}
            </select>
        </>
    }
}
