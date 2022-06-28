use crate::{
    components::input_range::InputRange,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use common::{
    granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction,
};
use log::info;
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsMinLen)]
pub fn controls_min_len() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let max_len_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let grain_len_min_value = app_context.state_handle.grain_len_min.get();
    let max = app_context.state_handle.get_buffer_len_ms();
    let min = GranularSynthesizer::GRAIN_LEN_ABSOLUTE_MIN_IN_MS;

    let handle_input = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let new_grain_len_min = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as u32;
            state_handle.dispatch(AppAction::SetGrainLenMin(new_grain_len_min));
        })
    };

    info!("grain_len_min_value (input) = {}", grain_len_min_value);

    html! {
        <InputRange
            label="min"
            id="controls-min-length"
            min={min.to_string()}
            max={max.to_string()}
            step="1"
            oninput={handle_input}
            value={grain_len_min_value.to_string()}
            disabled={max_len_input_disabled}
        />
    }
}
