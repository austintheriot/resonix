use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AudioContextComputeError {
    #[error("Not enough input data provided")]
    NotEnoughInputData,
}
