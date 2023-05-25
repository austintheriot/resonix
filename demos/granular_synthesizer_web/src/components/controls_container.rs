use crate::{
    components::{
        audio_output_visualization::AudioOutputVisualization, buffer_container::BufferContainer,
        controls_channels::ControlsChannels, controls_download_audio::ControlsDownloadAudio,
        controls_enable_audio::ControlsEnableAudio, controls_gain::ControlsGain,
        controls_len::ControlsLen, controls_play_status::ControlsPlayStatus,
        controls_recording_status::ControlsRecordingStatus,
        controls_delay::ControlsDelay, controls_reset::ControlsReset,
        controls_select_buffer::ControlsSelectBuffer, controls_upload_buffer::ControlsUploadBuffer,
        loading_indicator::LoadingIndicator,
    },
    state::app_context::{AppContext, AppContextError},
};
use yew::{classes, function_component, html, use_context};

#[function_component(ControlsContainer)]
pub fn controls_container() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let keyboard_user_class = app_context
        .state_handle
        .is_keyboard_user
        .then_some("keyboard-user");

    html! {
        <div class={classes!("controls-container", keyboard_user_class)}>
            <LoadingIndicator />
            <div class="grid-button-container">
                <ControlsEnableAudio />
                <ControlsPlayStatus />
                <ControlsReset />
                <ControlsRecordingStatus />
                <ControlsDownloadAudio />
            </div>
            <div class="grid-slider-container">
                <ControlsGain />
                <ControlsChannels />
                <ControlsLen />
                <ControlsDelay />
            </div>
            <div class="grid-select-container">
                <ControlsSelectBuffer />
                <ControlsUploadBuffer />
            </div>
            <div class="grid-buffer-container">
                <AudioOutputVisualization />
                <BufferContainer />
            </div>
        </div>
    }
}
