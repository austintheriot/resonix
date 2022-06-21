use std::sync::Arc;
use crate::audio::{stream_handle::StreamHandle, current_status::CurrentStatus};
use super::app_state::SampleRate;

#[derive(Debug, Clone)]
pub enum AppAction {
    /// If no `SampleRate` is supplied, the default `sample_rate` in
    /// the existing app state is used.
    SetBuffer(Arc<Vec<f32>>, Option<SampleRate>),
    SetStreamHandle(Option<StreamHandle>),
    SetBufferSelectionStart(f32),
    SetBufferSelectionEnd(f32),
    SetBufferSelectionMouseDown(bool),
    SetGain(f32),
    SetStatus(CurrentStatus),
    SetAudioInitialized(bool),
    SetAudioLoading(bool),
    SetSampleRate(u32),
    SetDensity(f32)
}