use crate::{AudioContextInterfaceData, AudioContextUid};

pub struct AudioContext {
    current_uid: usize,
}

impl AudioContext {
    fn get_incremented_uid(&mut self) -> usize {
        self.current_uid += 1;
        return self.current_uid;
    }
}

impl resonix_types::AudioContext for AudioContext {
    fn get_new_uid(&mut self) -> impl resonix_types::AudioContextUid {
        AudioContextUid(self.get_incremented_uid())
    }

    fn new_with_options(
        _options: impl resonix_types::AudioContextOptions,
    ) -> impl resonix_types::AudioContext {
        Self { current_uid: 0 }
    }

    fn new() -> impl resonix_types::AudioContext {
        Self { current_uid: 0 }
    }

    fn compute_next_frame(
        &mut self,
    ) -> Result<
        impl Iterator<Item = impl resonix_types::AudioContextInterfaceData>,
        resonix_types::AudioContextComputeError,
    > {
        Ok(std::iter::empty::<AudioContextInterfaceData>())
    }

    fn compute_next_frame_with_data(
        &mut self,
        interface_data: impl Iterator<Item = impl resonix_types::AudioContextInterfaceData>,
    ) -> Result<
        impl Iterator<Item = impl resonix_types::AudioContextInterfaceData>,
        resonix_types::AudioContextComputeError,
    > {
        todo!()
    }

    fn connect_ports(
        &mut self,
        this_port_address: impl resonix_types::AudioPortAddress,
        that_port_address: impl resonix_types::AudioPortAddress,
        num_channels: impl resonix_types::NumChannels,
    ) -> Result<&impl resonix_types::ConnectionDescriptor, resonix_types::ConnectionError> {
        todo!()
    }
}
