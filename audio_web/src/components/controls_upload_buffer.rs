use crate::{
    audio::{decode, play_status::PlayStatus},
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use js_sys::{ArrayBuffer, Uint8Array};
use log::info;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlInputElement};
use yew::{function_component, html, prelude::*};

#[function_component(ControlsUploadBuffer)]
pub fn controls_upload_buffer() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let input_ref = use_node_ref();
    let input_element_disabled = app_context.state_handle.get_are_audio_controls_disabled();

    let handle_change = {
        let state_handle = app_context.state_handle;
        let input_ref = input_ref.clone();
        Callback::from(move |_: Event| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let state_handle = state_handle.clone();
            let input_ref = input_ref.clone();
            wasm_bindgen_futures::spawn_local(async move {
                state_handle.dispatch(AppAction::SetAudioLoading(true));
                state_handle.dispatch(AppAction::SetPlayStatus(PlayStatus::Pause));

                let input_element = input_ref
                    .get()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();

                if let Some(files) = input_element.files() {
                    if let Some(file) = files.item(0) {
                        info!("Found a file! {:?}", file.name());

                        let file_array_buffer_promise = file.array_buffer();
                        let file_array_buffer: ArrayBuffer =
                            wasm_bindgen_futures::JsFuture::from(file_array_buffer_promise)
                                .await
                                .expect("Should be able to get array buffer from uploaded file")
                                .dyn_into()
                                .unwrap();
                        let file_array_buffer = Uint8Array::new(file_array_buffer.as_ref());
                        let file_bytes = file_array_buffer.to_vec();

                        // @todo: initialize a single audio_context at the top level of the app
                        let audio_context = web_sys::AudioContext::new()
                            .expect("Browser should have AudioContext implemented");
                        let audio_buffer_result =
                            decode::decode_bytes(&audio_context, &file_bytes).await;

                        match audio_buffer_result {
                            Ok(audio_buffer) => {
                                let buffer_data =
                                    Arc::new(audio_buffer.get_channel_data(0).unwrap());

                                state_handle.dispatch(AppAction::SetBuffer(buffer_data));
                            }
                            Err(_) => {
                                window()
                                    .unwrap()
                                    .alert_with_message("Error decoding uploaded audio file")
                                    .unwrap();
                            }
                        }
                    }
                }

                // regardless of success or failure, loading should be set to `false`
                state_handle.dispatch(AppAction::SetAudioLoading(false));
            })
        })
    };

    let disabled_class = if input_element_disabled {
        "disabled"
    } else {
        ""
    };

    html! {
        <div class={classes!("controls-upload-buffer", disabled_class)}>
            <label for="controls-upload-buffer">
                {"Select File"}
            </label>
            <input
                id="controls-upload-buffer"
                type="file"
                accept="audio/*"
                onchange={handle_change}
                ref={input_ref}
                disabled={input_element_disabled}
            />
        </div>
    }
}
