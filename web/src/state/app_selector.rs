use super::app_state::AppState;

pub trait AppSelector {
    fn get_are_audio_controls_disabled(&self) -> bool;
    fn get_buffer_len_ms(&self) -> usize;
}

impl AppSelector for AppState {
    fn get_are_audio_controls_disabled(&self) -> bool {
        !self.audio_initialized || self.audio_loading
    }

    fn get_buffer_len_ms(&self) -> usize {
        if self.sample_rate == 0 {
            return 0;
        }

        self.buffer_handle.get_data().len() / self.sample_rate as usize
    }
}
