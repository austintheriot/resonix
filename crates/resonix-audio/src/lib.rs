#![no_std]

extern crate alloc;

mod cpal;
mod mock;
mod shared;

#[cfg(feature = "cpal")]
pub use cpal::*;
#[cfg(feature = "mock")]
pub use mock::*;
pub use shared::*;
