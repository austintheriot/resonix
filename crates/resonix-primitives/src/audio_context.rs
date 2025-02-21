pub struct AudioContext {
    current_uid: usize,
}

impl AudioContext {
    fn get_incremented_usize(&mut self) -> usize {
        self.current_uid += 1;
        return self.current_uid;
    }
}

impl<
        A: resonix_types::Amplitude,
        S: resonix_types::Sample<A>,
        F: resonix_types::Frame<A, S>,
        U: resonix_types::AudioContextUid,
        I: resonix_types::AudioContextInterfaceData<A, S, F, U>,
        N: resonix_types::NumChannels,
        P: resonix_types::AudioPortAddress,
        D: resonix_types::ConnectionDescriptor<N, U, P>,
    > resonix_types::AudioContext<A, S, F, U, I, N, P, D> for AudioContext
{
    fn get_new_uid(&mut self) -> U {
        U::from_usize(self.get_incremented_usize())
    }

    fn new_with_options(_options: impl resonix_types::AudioContextOptions) -> Self {
        Self { current_uid: 0 }
    }

    fn new() -> Self {
        Self { current_uid: 0 }
    }

    fn compute_next_frame(&mut self) -> Result<Vec<I>, resonix_types::AudioContextComputeError> {
        todo!()
    }

    fn compute_next_frame_with_data(
        &mut self,
        _interface_data: impl Iterator<Item = impl resonix_types::AudioContextInterfaceData<A, S, F, U>>,
    ) -> Result<Vec<I>, resonix_types::AudioContextComputeError> {
        todo!()
    }

    fn connect_ports(
        &mut self,
        this_port_address: impl resonix_types::AudioPortAddress,
        that_port_address: impl resonix_types::AudioPortAddress,
        num_channels: impl resonix_types::NumChannels,
    ) -> Result<&D, resonix_types::ConnectionError> {
        todo!()
    }
}
