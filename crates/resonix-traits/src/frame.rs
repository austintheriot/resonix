use crate::Sample;

pub trait Frame {
    fn samples(&self) -> &[impl Sample];
}
