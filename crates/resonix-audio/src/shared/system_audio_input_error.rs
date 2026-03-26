use thiserror::Error;

use crate::ConsumerError;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SystemAudioInputError {
    #[error("Consumer error occurred: {0:?}")]
    ConsumerError(#[from] ConsumerError),
}
