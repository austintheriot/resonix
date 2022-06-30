use crate::{
    components::input_range::InputRange,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use audio_common::{
    granular_synthesizer::GranularSynthesizer, granular_synthesizer_action::GranularSynthesizerAction,
};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsRefreshInterval)]
pub fn controls_refresh_interval() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let refresh_interval_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let refresh_interval = app_context.state_handle.refresh_interval.get();

    let handle_input = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let refresh_interval = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as u32;
            state_handle.dispatch(AppAction::SetRefreshInterval(refresh_interval));
        })
    };

    html! {
        <InputRange
            label="fade"
            id="controls-refresh-interval"
            min={GranularSynthesizer::REFRESH_INTERVAL_MIN.to_string()}
            max={GranularSynthesizer::REFRESH_INTERVAL_MAX.to_string()}
            step="1"
            oninput={handle_input}
            value={refresh_interval.to_string()}
            disabled={refresh_interval_input_disabled}
        />
    }
}
