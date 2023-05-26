use crate::{
    audio::num_channels_action::NumChannelsAction,
    components::input_range::{GetLabelCallback, InputRange},
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
    let channels = app_context.state_handle.num_channels_handle.get().get();

    let handle_input = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: InputEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }

            let channels = e
                .target_dyn_into::<HtmlInputElement>()
                .unwrap()
                .value_as_number() as usize;
            state_handle.dispatch(AppAction::SetNumSynthChannels(channels));
        })
    };

    let get_label_on_input = |value: f64| -> Option<String> { Some(format!("{}", value as u32)) };

    html! {
        <InputRange
            label="channels"
            id="controls-num-channels-input"
            min="0"
            max="100"
            step="1"
            oninput={handle_input}
            value={channels.to_string()}
            disabled={density_input_disabled}
            get_label_on_input={Into::<GetLabelCallback>::into(get_label_on_input)}
        />
    }
}
