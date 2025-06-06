use alloc::boxed::Box;
use core::error::Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemAudioOutputError {
    #[error("Internal buffer error write occurred: {0}")]
    WriteError(#[from] Box<dyn Error>),
}
