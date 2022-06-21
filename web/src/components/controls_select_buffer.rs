use crate::{
    state::{
        app_context::{AppContext, AppContextError},
    },
};
use yew::{function_component, html, prelude::*};

// this list of audio files is generated statically at build time
// so that the list of default audio files always matches the actual list
// of files available from the `audio` directory
include!(concat!(env!("OUT_DIR"), "/audio_files.rs"));

#[function_component(ControlsSelectBuffer)]
pub fn controls_select_buffer() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);

    let handle_change = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: Event| {
        })
    };

    html! {
        <>
            <label for="controls-select-buffer">
                {"Play"}
            </label>
            <select
                id="controls-select-buffer"
                class="controls-select-buffer"
                onchange={handle_change}
            >
                {AUDIO_FILES.iter().map(|file_name| {
                    html!{
                        <option>
                            {file_name}
                        </option>
                    }
                }).collect::<Html>()}
            </select>
        </>
    }
}
