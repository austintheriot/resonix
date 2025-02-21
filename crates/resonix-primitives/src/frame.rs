use std::marker::PhantomData;

use crate::{Amplitude, Sample};

pub struct Frame<A: resonix_types::Amplitude = Amplitude, S: resonix_types::Sample<A> = Sample<A>> {
    samples: Vec<S>,
    amplitude_type: PhantomData<A>,
}

impl<A: resonix_types::Amplitude, S: resonix_types::Sample<A>> resonix_types::Frame<A, S>
    for Frame<A, S>
{
    fn samples(&self) -> &[S] {
        self.samples.as_slice()
    }

    fn into_samples(self) -> Vec<S> {
        self.samples
    }
}
