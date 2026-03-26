use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("No data to read")]
    NoData,
}
