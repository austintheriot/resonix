use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use alloc::boxed::Box;

use crate::{primitives::ConnectionId, utils::IntMap};

use super::Sample;

// TODO: refactor into multichannel Buffer eventually
//
// The slice is wrapped in UnsafeCell so that raw pointers derived from
// `UnsafeCell::get()` carry SharedReadWrite (SRW) provenance under Stacked
// Borrows.  SRW tags live at the base of the borrow stack and are never
// invalidated by Unique retags from mutable accesses, which is required for
// the compiled-plan execution model where one node writes a buffer and a
// later node reads it through independently-derived raw pointers.
pub type AudioBuffer = Box<UnsafeCell<[Sample]>>;

#[derive(Debug, Default)]
pub struct BufferPool {
    buffers: IntMap<ConnectionId, AudioBuffer>,
}

impl Deref for BufferPool {
    type Target = IntMap<ConnectionId, AudioBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.buffers
    }
}

impl DerefMut for BufferPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffers
    }
}
