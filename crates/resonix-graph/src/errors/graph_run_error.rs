use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphRunError {
    #[error("visit order unexpectedly included the id of a non-node value")]
    VisitOrderIncludedNonNodeValue,
    #[error("node unexpectedly returned an error when processing inputs")]
    NodeReturnedError,
}
