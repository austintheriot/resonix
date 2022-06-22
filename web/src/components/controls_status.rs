use crate::{
    audio::{current_status::CurrentStatus, current_status_action::CurrentStatusAction},
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use web_sys::HtmlInputElement;
use yew::{function_component, html, prelude::*};

#[function_component(ControlsStatus)]
pub fn controls_status() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let current_status = app_context.state_handle.current_status_handle.get();
    let input_disabled = app_context.state_handle.get_are_audio_controls_disabled();

    let handle_change = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: InputEvent| {
            let checked = e.target_dyn_into::<HtmlInputElement>().unwrap().checked();
            let current_status = match checked {
                true => CurrentStatus::PLAY,
                false => CurrentStatus::PAUSE,
            };
            state_handle.dispatch(AppAction::SetStatus(current_status));
        })
    };

    html! {
        <>
            <label for="controls-status">
                {"Play"}
            </label>
            <input
                id="controls-status"
                class="controls-status"
                type="checkbox"
                oninput={handle_change}
                checked={current_status.into()}
                disabled={input_disabled}
            />
        </>
    }
}
