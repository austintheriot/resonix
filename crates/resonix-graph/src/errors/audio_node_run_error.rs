use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioNodeRunError {
    #[error("too many inputs (expected {expected:?}, found {found:?})")]
    TooManyInputs { expected: usize, found: usize },
}
