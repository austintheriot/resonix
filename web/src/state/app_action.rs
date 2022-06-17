use std::sync::Arc;

use crate::audio::stream_handle::StreamHandle;

#[derive(Debug, Clone)]
pub enum AppAction {
    SetBuffer(Arc<Vec<f32>>),
    SetStreamHandle(Option<StreamHandle>)
}
