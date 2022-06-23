use crate::{
    audio::density_action::DensityAction,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsDensity)]
pub fn controls_density() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let density_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let density = app_context.state_handle.density_handle.get();

    let handle_change = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            let density = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as f32;
            state_handle.dispatch(AppAction::SetDensity(density));
        })
    };

    html! {
        <>
            <label for="controls-density">
                {"Density"}
            </label>
            <input
                id="controls-density"
                class="controls-density"
                type="range"
                min={0.0}
                max={1.0}
                step={0.001}
                oninput={handle_change}
                value={density.to_string()}
                disabled={density_input_disabled}
            />
        </>
    }
}
