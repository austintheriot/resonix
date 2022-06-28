use crate::{
    components::buffer_container::BufferContainer, components::controls_density::ControlsDensity,
    components::controls_enable_audio::ControlsEnableAudio,
    components::controls_gain::ControlsGain, components::controls_play_status::ControlsPlayStatus,
    components::controls_select_buffer::ControlsSelectBuffer,
    components::controls_min_len::ControlsMinLen,
    components::controls_max_len::ControlsMaxLen,
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
            </div>
            <ControlsSelectBuffer />
            <BufferContainer />
        </div>
    }
}
