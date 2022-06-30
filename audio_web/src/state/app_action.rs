use crate::audio::{play_status::PlayStatus, stream_handle::StreamHandle};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AppAction {
    SetBuffer(Arc<Vec<f32>>),
    SetStreamHandle(Option<StreamHandle>),
    SetBufferSelectionStart(f32),
    SetBufferSelectionEnd(f32),
    SetBufferSelectionMouseDown(bool),
    SetGain(f32),
    SetPlayStatus(PlayStatus),
    SetAudioInitialized(bool),
    SetAudioLoading(bool),
    SetSampleRate(u32),
    SetDensity(f32),
    SetGrainLenMax(f32),
    SetGrainLenMin(f32),
    SetRefreshInterval(u32),
    ResetState,
}
