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

#[function_component(ControlsMaxLen)]
pub fn controls_max_len() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let max_len_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let grain_len_max_value = app_context.state_handle.grain_len_max.get().get();

    let handle_input = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let new_grain_len_max = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as f32;
            state_handle.dispatch(AppAction::SetGrainLenMax(new_grain_len_max));
        })
    };

    html! {
        <InputRange
            label="max\nlen"
            id="controls-max-length"
            min={GranularSynthesizer::GRAIN_LEN_MAX_MIN.to_string()}
            max={GranularSynthesizer::GRAIN_LEN_MAX_MAX.to_string()}
            step="0.01"
            oninput={handle_input}
            value={grain_len_max_value.to_string()}
            disabled={max_len_input_disabled}
        />
    }
}
