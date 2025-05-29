use thiserror::Error;

use super::AudioNodeRunError;

#[derive(Error, Debug)]
pub enum GraphRunError {
    #[error("visit order unexpectedly included the id of a non-node value")]
    VisitOrderIncludedNonNodeValue,
    #[error("node unexpectedly returned an error when processing inputs")]
    NodeReturnedError,
    #[error("audio node encountered error while processing graph")]
    AudioNodeRunError(#[from] AudioNodeRunError),
}
