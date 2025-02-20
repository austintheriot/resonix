use crate::{Amplitude, Sample};

pub struct Frame<A: resonix_types::Amplitude, S: resonix_types::Sample = Sample<Amplitude>> {
    samples: Vec<S>,
}

impl<A: resonix_types::Amplitude, S: resonix_types::Sample> resonix_types::Frame<A, S>
    for Frame<A, S>
{
    fn samples(&self) -> &[S] {
        self.samples.as_slice()
    }

    fn into_samples(self) -> Vec<S> {
        self.samples
    }
}
