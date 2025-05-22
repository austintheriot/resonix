pub trait HasPortDescriptors {
    type PortDescriptors;

    fn port_descriptors(&self) -> Self::PortDescriptors;
}
