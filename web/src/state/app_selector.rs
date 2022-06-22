use super::app_state::AppState;

pub trait AppSelector {
    fn get_is_enable_audio_button_disabled(&self) -> bool;
    fn get_are_audio_controls_disabled(&self) -> bool;
}

impl AppSelector for AppState {
    fn get_is_enable_audio_button_disabled(&self) -> bool {
        self.audio_initialized || self.audio_loading
    }

    fn get_are_audio_controls_disabled(&self) -> bool {
        !self.audio_initialized
    }
}
