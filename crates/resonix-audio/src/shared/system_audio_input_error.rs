use alloc::boxed::Box;
use core::error::Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemAudioInputError {
    #[error("Internal buffer error read occurred: {0}")]
    ReadError(#[from] Box<dyn Error>),
}
