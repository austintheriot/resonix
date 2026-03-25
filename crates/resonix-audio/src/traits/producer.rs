use crate::ProducerError;

pub trait Producer<S> {
    fn try_write(&mut self, value: S) -> Result<(), ProducerError>;

    fn ready(&self) -> bool;
}
