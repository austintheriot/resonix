use crate::{
    audio::channels_action::ChannelsAction,
    components::input_range::InputRange,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsChannels)]
pub fn controls_density() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let density_input_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let channels = app_context.state_handle.channels_handle.get().get();

    let handle_input = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let channels = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as f32;
            state_handle.dispatch(AppAction::SetChannels(channels));
        })
    };

    html! {
        <InputRange
            label="channels"
            id="controls-channels-input"
            min="0.0"
            max="1.0"
            step="0.0001"
            oninput={handle_input}
            value={channels.to_string()}
            disabled={density_input_disabled}
        />
    }
}