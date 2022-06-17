use std::sync::Arc;

use cpal::{Stream};
use uuid::Uuid;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt::Debug;

/// A wrapper around `cpal`'s Stream type for implementing `PartialEq`, etc.
#[wasm_bindgen]
#[derive(Clone)]
pub struct StreamHandle {
    stream: Arc<Stream>,
    uuid: Uuid
}

impl PartialEq for StreamHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Debug for StreamHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamHandle").field("uuid", &self.uuid).finish()
    }
}

impl StreamHandle {
    pub fn new(stream: Stream) -> Self {
        StreamHandle { stream: Arc::new(stream), uuid: Uuid::new_v4() }
    }
}