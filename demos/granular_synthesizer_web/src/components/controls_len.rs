use std::time::Duration;

use crate::{
    components::input_range::InputRange,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use audio::{
    granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction,
};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsLen)]
pub fn controls_len() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let max_len_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let grain_len = app_context.state_handle.grain_len.get();

    let handle_input = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let new_grain_len = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as u64;
            let new_grain_len = Duration::from_millis(new_grain_len);
            state_handle.dispatch(AppAction::SetGrainLen(new_grain_len));
        })
    };

    html! {
        <InputRange
            label="len (ms)"
            id="controls-grain-length"
            min={GranularSynthesizer::GRAIN_LEN_MIN.as_millis().to_string()}
            max={GranularSynthesizer::GRAIN_LEN_MAX.as_millis().to_string()}
            step="1"
            oninput={handle_input}
            value={grain_len.as_millis().to_string()}
            disabled={max_len_input_disabled}
        />
    }
}
