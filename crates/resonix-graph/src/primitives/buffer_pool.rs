use core::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use alloc::boxed::Box;

use crate::{primitives::ConnectionId, utils::IntMap};

use super::Sample;

// TODO: refactor into multichannel Buffer eventually
pub type AudioBuffer = Box<[Sample]>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BufferPool {
    // strictly speaking, we don't NEED a `RefCell` here,
    // but it does give some peace of mind for the limited
    // amount of `unsafe` we use to reference these buffers
    // later in the `process` call
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
