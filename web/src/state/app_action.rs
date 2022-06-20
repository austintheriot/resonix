use std::sync::Arc;
use crate::audio::{stream_handle::StreamHandle, current_status::CurrentStatus};

#[derive(Debug, Clone)]
pub enum AppAction {
    SetBuffer(Arc<Vec<f32>>),
    SetStreamHandle(Option<StreamHandle>),
    SetBufferSelectionStart(f32),
    SetBufferSelectionEnd(f32),
    SetBufferSelectionMouseDown(bool),
    SetGain(f32),
    SetStatus(CurrentStatus),
    SetAudioInitialized(bool),
}