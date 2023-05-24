use super::{app_action::AppAction, app_state::AppState};
use crate::{
    audio::{
        buffer_handle::BufferHandle, buffer_selection_action::BufferSelectionAction,
        channels_action::ChannelsAction, gain_action::GainAction,
        play_status_action::PlayStatusAction, recording_status_action::RecordingStatusAction,
    },
    components::buffer_sample_bars_canvas::get_buffer_maxes_for_canvas,
};
use audio::granular_synthesizer_action::GranularSynthesizerAction;
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
                    next_state.buffer_maxes_for_canvas = get_buffer_maxes_for_canvas(&buffer);
                    next_state
                        .granular_synthesizer_handle
                        .set_buffer(Arc::clone(&buffer));
                    next_state.buffer_handle = BufferHandle::new(buffer);
                }
                AppAction::SetStreamHandle(stream_handle) => {
                    // make sure previous state's stream handle gets dropped
                    next_state.stream_handle.take();
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
                AppAction::SetChannels(channels) => {
                    next_state.channels_handle.set(channels);
                    next_state
                        .granular_synthesizer_handle
                        .set_channels(channels);
                }
                AppAction::SetGrainLenMax(max_len) => {
                    next_state
                        .granular_synthesizer_handle
                        .set_grain_len_max(max_len);

                    // keep ui state in sync with synthesizer
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

                    // keep ui state in sync with synthesizer
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
                    // drop previous stream's handle to stop audio
                    next_state.stream_handle.take();
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
                AppAction::SetRecordingStatus(recording_status) => {
                    next_state.recording_status_handle.set(recording_status);
                }
                AppAction::SetNumChannels(num_channels) => {
                    next_state.num_channels = num_channels;
                }
                AppAction::DownloadAudio => next_state
                    .audio_recorder_handle
                    .download_as_wav(next_state.num_channels as u16, next_state.sample_rate),
                AppAction::SetIsKeyboardUser => {
                    next_state.is_keyboard_user = true;
                }
            }
        }

        Rc::new(next_state)
    }
}
