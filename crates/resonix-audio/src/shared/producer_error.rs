use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("Not enought space to write data to.")]
    InsufficientSpace,
}
