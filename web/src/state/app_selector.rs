use super::app_state::AppState;

pub trait AppSelector {
    fn get_is_enable_audio_button_disabled(&self) -> bool;
    fn get_is_play_input_disabled(&self) -> bool;
    fn get_is_buffer_selection_visualizer_disabled(&self) -> bool;
    fn get_is_gain_input_disabled(&self) -> bool;
    fn get_is_density_input_disabled(&self) -> bool;
}

impl AppSelector for AppState {
    fn get_is_enable_audio_button_disabled(&self) -> bool {
        self.audio_initialized || self.audio_loading
    }

    fn get_is_play_input_disabled(&self) -> bool {
        !self.audio_initialized
    }

    fn get_is_buffer_selection_visualizer_disabled(&self) -> bool {
        !self.audio_initialized
    }

    fn get_is_gain_input_disabled(&self) -> bool {
        !self.audio_initialized
    }

    fn get_is_density_input_disabled(&self) -> bool {
        !self.audio_initialized
    }
}
