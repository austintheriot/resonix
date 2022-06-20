use crate::{
    audio::gain::GAIN_MAX,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
    },
};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(GainControl)]
pub fn gain_control() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let gain = app_context.state_handle.gain.get();

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
            <label for="gain-control">
                {"Gain"}
            </label>
            <input
                id="gain-control"
                class="gain-control"
                type="range"
                min={0.0}
                max={GAIN_MAX.to_string()}
                step={0.001}
                oninput={handle_change}
                value={gain.to_string()}
            />
        </>
    }
}
