use super::{app_action::AppAction, app_state::AppState};
use crate::{
    audio::{
        buffer_handle::BufferHandle, buffer_selection_action::BufferSelectionAction,
        density_action::DensityAction, gain_action::GainAction,
        play_status_action::PlayStatusAction,
    },
    components::buffer_sample_bars::get_buffer_maxes,
};
use audio_common::granular_synthesizer_action::GranularSynthesizerAction;
use std::{rc::Rc, sync::Arc};
use yew::Reducible;

pub const KEYBOARD_BUFFER_SELECTION_INCREMENT: f32 = 0.01;

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next_state = (*self).clone();
        {
            let action = action;
            match action {
                AppAction::SetBuffer(buffer) => {
                    next_state.buffer_maxes = get_buffer_maxes(&buffer);
                    next_state
                        .granular_synthesizer_handle
                        .set_buffer(Arc::clone(&buffer));
                    next_state.buffer_handle = BufferHandle::new(buffer);
                }
                AppAction::SetStreamHandle(stream_handle) => {
                    next_state.stream_handle = stream_handle;
                }
                AppAction::SetBufferSelectionStart(start) => {
                    next_state.buffer_selection_handle.set_mouse_start(start);
                }
                AppAction::SetBufferSelectionEnd(end) => {
                    next_state.buffer_selection_handle.set_mouse_end(end);
                }
                AppAction::SetBufferSelectionMouseDown(mouse_down) => {
                    next_state
                        .buffer_selection_handle
                        .set_mouse_down(mouse_down);
                }
                AppAction::SetGain(gain) => {
                    next_state.gain_handle.set(gain);
                }
                AppAction::SetPlayStatus(play_status) => {
                    next_state.play_status_handle.set(play_status);
                }
                AppAction::SetAudioInitialized(is_initialized) => {
                    next_state.audio_initialized = is_initialized;
                }
                AppAction::SetAudioLoading(loading) => {
                    next_state.audio_loading = loading;
                }
                AppAction::SetSampleRate(sample_rate) => {
                    next_state.sample_rate = sample_rate;

                    next_state
                        .granular_synthesizer_handle
                        .set_sample_rate(sample_rate);
                }
                AppAction::SetDensity(density) => {
                    next_state.density_handle.set(density);
                    next_state.granular_synthesizer_handle.set_density(density);
                }
                AppAction::SetGrainLenMax(max_len) => {
                    next_state
                        .granular_synthesizer_handle
                        .set_grain_len_max(max_len);
                    next_state
                        .grain_len_min
                        .set(next_state.granular_synthesizer_handle.grain_len_min());
                    next_state
                        .grain_len_max
                        .set(next_state.granular_synthesizer_handle.grain_len_max());
                }
                AppAction::SetGrainLenMin(min_len) => {
                    next_state
                        .granular_synthesizer_handle
                        .set_grain_len_min(min_len);
                    next_state
                        .grain_len_min
                        .set(next_state.granular_synthesizer_handle.grain_len_min());
                    next_state
                        .grain_len_max
                        .set(next_state.granular_synthesizer_handle.grain_len_max());
                }
                AppAction::SetRefreshInterval(refresh_interval) => {
                    next_state
                        .granular_synthesizer_handle
                        .set_refresh_interval(refresh_interval);
                    next_state
                        .refresh_interval
                        .set(next_state.granular_synthesizer_handle.refresh_interval());
                }
                AppAction::ResetState => {
                    next_state = AppState::default();
                }
                AppAction::IncrementBufferSelectionStart => {
                    next_state.buffer_selection_handle.set_mouse_start(
                        next_state.buffer_selection_handle.get_mouse_start()
                            + KEYBOARD_BUFFER_SELECTION_INCREMENT,
                    );
                    next_state
                        .buffer_selection_handle
                        .set_mouse_start(next_state.buffer_selection_handle.get_mouse_start());
                }
                AppAction::DecrementBufferSelectionStart => {
                    next_state.buffer_selection_handle.set_mouse_start(
                        next_state.buffer_selection_handle.get_mouse_start()
                            - KEYBOARD_BUFFER_SELECTION_INCREMENT,
                    );
                    next_state
                        .buffer_selection_handle
                        .set_mouse_start(next_state.buffer_selection_handle.get_mouse_start());
                }
                AppAction::IncrementBufferSelectionEnd => {
                    next_state.buffer_selection_handle.set_mouse_end(
                        next_state.buffer_selection_handle.get_mouse_end()
                            + KEYBOARD_BUFFER_SELECTION_INCREMENT,
                    );
                    next_state
                        .buffer_selection_handle
                        .set_mouse_end(next_state.buffer_selection_handle.get_mouse_end());
                }
                AppAction::DecrementBufferSelectionEnd => {
                    next_state.buffer_selection_handle.set_mouse_end(
                        next_state.buffer_selection_handle.get_mouse_end()
                            - KEYBOARD_BUFFER_SELECTION_INCREMENT,
                    );
                    next_state
                        .buffer_selection_handle
                        .set_mouse_end(next_state.buffer_selection_handle.get_mouse_end());
                }
            }
        }

        Rc::new(next_state)
    }
}
