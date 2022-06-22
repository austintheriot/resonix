use crate::{state::{
    app_action::AppAction,
    app_context::{AppContext, AppContextError},
    app_selector::AppSelector,
}, audio::{gain::Gain, gain_action::GainAction}};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsGain)]
pub fn controls_gain() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let gain_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let gain = app_context.state_handle.gain_handle.get();

    let handle_change = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: InputEvent| {
            let gain = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as f32;
            state_handle.dispatch(AppAction::SetGain(gain));
        })
    };

    html! {
        <>
            <label for="controls-gain">
                {"Gain"}
            </label>
            <input
                id="controls-gain"
                class="controls-gain"
                type="range"
                min={Gain::GAIN_MIN.to_string()}
                max={Gain::GAIN_MAX.to_string()}
                step={0.001}
                oninput={handle_change}
                value={gain.to_string()}
                disabled={gain_input_disabled}
            />
        </>
    }
}
