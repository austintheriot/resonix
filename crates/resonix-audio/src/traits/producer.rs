use crate::ProducerError;

pub trait Producer<V> {
    fn try_write(&mut self, value: V) -> Result<(), ProducerError>;
}
