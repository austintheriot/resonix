use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ConnectionError {
    #[error("Node does not accept input connections")]
    CantAcceptInput,
    #[error("Node does not accept output connections")]
    CantAcceptOutput,
}
