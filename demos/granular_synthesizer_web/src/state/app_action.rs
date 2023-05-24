use crate::audio::{
    play_status::PlayStatus, recording_status::RecordingStatus, stream_handle::StreamHandle,
};
use std::sync::Arc;

use super::app_state::NunChannels;

#[derive(Debug, Clone)]
pub enum AppAction {
    SetBuffer(Arc<Vec<f32>>),
    SetStreamHandle(StreamHandle),
    SetBufferSelectionStart(f32),
    IncrementBufferSelectionStart,
    DecrementBufferSelectionStart,
    IncrementBufferSelectionEnd,
    DecrementBufferSelectionEnd,
    SetBufferSelectionEnd(f32),
    SetBufferSelectionMouseDown(bool),
    SetGain(f32),
    SetPlayStatus(PlayStatus),
    SetAudioInitialized(bool),
    SetAudioLoading(bool),
    SetSampleRate(u32),
    SetNumSynthChannels(usize),
    SetGrainLenMax(f32),
    SetGrainLenMin(f32),
    SetRefreshInterval(u32),
    ResetState,
    SetRecordingStatus(RecordingStatus),
    SetNumChannels(NunChannels),
    DownloadAudio,
    SetIsKeyboardUser,
}
