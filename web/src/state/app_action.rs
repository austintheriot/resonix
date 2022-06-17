use std::sync::Arc;
use serde::{Serialize, ser::SerializeTupleVariant};
use crate::audio::stream_handle::StreamHandle;

#[derive(Debug, Clone)]
pub enum AppAction {
    SetBuffer(Arc<Vec<f32>>),
    SetStreamHandle(Option<StreamHandle>),
    SetBufferSelectionStart(f32),
    SetBufferSelectionEnd(f32),
    SetBufferSelectionMouseDown(bool)
}

/// This is only serialized for state update logging purposes
impl Serialize for AppAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            match self {
                AppAction::SetBuffer(vec) => {
                    let mut state = serializer.serialize_tuple_variant("SetBuffer", 0, "SetBuffer", 1)?;
                    state.serialize_field(&(**vec))?;
                    state.end()
                }
                AppAction::SetStreamHandle(stream_handle) => {
                    let mut state = serializer.serialize_tuple_variant("SetStreamHandle", 1, "SetStreamHandle", 1)?;
                    state.serialize_field(stream_handle)?;
                    state.end()
                },
                AppAction::SetBufferSelectionStart(start) => {
                    let mut state = serializer.serialize_tuple_variant("SetBufferSelectionStart", 2, "SetBufferSelectionStart", 1)?;
                    state.serialize_field(start)?;
                    state.end()
                },
                AppAction::SetBufferSelectionEnd(end) => {
                    let mut state = serializer.serialize_tuple_variant("SetBufferSelectionEnd", 3, "SetBufferSelectionEnd", 1)?;
                    state.serialize_field(end)?;
                    state.end()
                },
                AppAction::SetBufferSelectionMouseDown(is_down) => {
                    let mut state = serializer.serialize_tuple_variant("SetBufferSelectionMouseDown", 4, "SetBufferSelectionMouseDown", 1)?;
                    state.serialize_field(is_down)?;
                    state.end()
                },
            }
    }
}