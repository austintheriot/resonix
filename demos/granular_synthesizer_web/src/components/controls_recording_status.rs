use super::button::ButtonVariant;
use crate::{
    audio::{recording_status::RecordingStatus, recording_status_action::RecordingStatusAction},
    components::button::Button,
    icons::record::IconRecord,
    icons::stop_recording::IconStopRecording,
    state::{
        app_action::AppAction,
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
};
use yew::{function_component, html, prelude::*};

#[function_component(ControlsRecordingStatus)]
pub fn controls_recording_status() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let current_recording_status = app_context.state_handle.recording_status_handle.get();
    let button_disabled = app_context.state_handle.get_are_audio_controls_disabled();

    let handle_record = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }
            match state_handle.recording_status_handle.get() {
                RecordingStatus::Recording => {
                    state_handle.dispatch(AppAction::SetRecordingStatus(RecordingStatus::Stop))
                }
                RecordingStatus::Stop => {
                    state_handle.dispatch(AppAction::SetRecordingStatus(RecordingStatus::Recording))
                }
            }
        })
    };

    let handle_pause = {
        let state_handle = app_context.state_handle;
        Callback::from(move |_: MouseEvent| {
            if state_handle.get_are_audio_controls_disabled() {
                return;
            }
            state_handle.dispatch(AppAction::SetRecordingStatus(RecordingStatus::Stop));
        })
    };

    let default_class = "controls-recording-status";

    match current_recording_status {
        RecordingStatus::Recording => html! {
            <Button
                aria_label="stop recording audio"
                class={classes!(default_class, "stop")}
                onclick={handle_pause}
                disabled={button_disabled}
                variant={ButtonVariant::Pressed}
            >
                <IconStopRecording />
            </Button>
        },
        RecordingStatus::Stop => html! {
            <Button
                aria_label="record audio"
                class={classes!(default_class, "record")}
                onclick={handle_record}
                disabled={button_disabled}
                variant={ButtonVariant::Unpressed}
            >
                <IconRecord />
            </Button>
        },
    }
}
