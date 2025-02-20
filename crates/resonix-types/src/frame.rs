use crate::Sample;

pub trait Frame {
    fn samples(&self) -> &[impl Sample];

    fn into_samples(self) -> impl Iterator<Item = impl Sample>;
}
