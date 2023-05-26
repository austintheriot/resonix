use std::time::Duration;

use crate::{
    components::input_range::{GetLabelCallback, InputRange},
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

#[function_component(ControlsDelay)]
pub fn controls_delay() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let delay_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let delay = app_context.state_handle.grain_initialization_delay.get();

    let handle_input = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let delay_ms = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as u64;
            state_handle.dispatch(AppAction::SetGrainInitializationDelay(
                Duration::from_millis(delay_ms),
            ));
        })
    };

    let get_label_on_input = |value: f64| -> Option<String> { Some(format!("{}ms", value as u32)) };

    html! {
        <InputRange
            label="delay"
            id="controls-refresh-interval"
            min={GranularSynthesizer::GRAIN_INITIALIZATION_DELAY_MIN.as_millis().to_string()}
            max={GranularSynthesizer::GRAIN_INITIALIZATION_DELAY_MAX.as_millis().to_string()}
            step="1"
            oninput={handle_input}
            value={delay.as_millis().to_string()}
            disabled={delay_input_disabled}
            get_label_on_input={Into::<GetLabelCallback>::into(get_label_on_input)}
        />
    }
}
