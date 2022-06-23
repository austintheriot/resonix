use crate::{
    components::buffer_container::BufferContainer, components::controls_density::ControlsDensity,
    components::controls_enable_audio::ControlsEnableAudio,
    components::controls_gain::ControlsGain, components::controls_play_status::ControlsPlayStatus,
    components::controls_select_buffer::ControlsSelectBuffer,
};
use yew::{function_component, html};

#[function_component(ControlsContainer)]
pub fn controls_container() -> Html {
    html! {
        <div class="controls-container">
            <ControlsEnableAudio />
            <ControlsPlayStatus />
            <ControlsGain />
            <ControlsDensity />
            <ControlsSelectBuffer />
            <BufferContainer />
        </div>
    }
}
