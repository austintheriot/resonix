use crate::AudioPortAddress;

pub trait AudioPortDescriptor {
    fn name(&self) -> &str;

    fn address(&self) -> &impl AudioPortAddress;
}
