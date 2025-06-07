#![no_std]

extern crate alloc;

mod cpal_impl;
mod mock_impl;
mod shared;

#[cfg(feature = "cpal")]
pub use cpal::{Sample, SizedSample};
#[cfg(feature = "cpal")]
pub use cpal_impl::*;
#[cfg(feature = "mock")]
pub use mock_impl::*;
pub use shared::*;
