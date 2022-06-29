use crate::components::{
    buffer_container::BufferContainer, controls_density::ControlsDensity,
    controls_enable_audio::ControlsEnableAudio, controls_gain::ControlsGain,
    controls_max_len::ControlsMaxLen, controls_min_len::ControlsMinLen,
    controls_play_status::ControlsPlayStatus, controls_refresh_interval::ControlsRefreshInterval,
    controls_select_buffer::ControlsSelectBuffer,
};
use yew::{function_component, html};

#[function_component(ControlsContainer)]
pub fn controls_container() -> Html {
    html! {
        <div class="controls-container">
            <div class="grid-button-container">
                <ControlsEnableAudio />
                <ControlsPlayStatus />
            </div>
            <div class="grid-slider-container">
                <ControlsGain />
                <ControlsDensity />
                <ControlsMinLen />
                <ControlsMaxLen />
                <ControlsRefreshInterval />
            </div>
            <ControlsSelectBuffer />
            <BufferContainer />
        </div>
    }
}
