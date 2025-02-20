use crate::{Amplitude, Sample};

pub trait Frame<A: Amplitude, S: Sample<A>> {
    fn samples(&self) -> &[S];

    fn into_samples(self) -> Vec<S>;
}
