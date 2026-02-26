use core::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use alloc::boxed::Box;

use crate::{primitives::ConnectionId, utils::IntMap};

// TODO: refactor into multichannel Buffer eventually
pub type AudioBuffer = Box<[f32]>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BufferPool {
    buffers: IntMap<ConnectionId, RefCell<AudioBuffer>>,
}

impl Deref for BufferPool {
    type Target = IntMap<ConnectionId, RefCell<AudioBuffer>>;

    fn deref(&self) -> &Self::Target {
        &self.buffers
    }
}

impl DerefMut for BufferPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffers
    }
}
