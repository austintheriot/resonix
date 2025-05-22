pub trait HasPortDescriptors<PortDescriptors> {
    fn port_descriptors(&self) -> PortDescriptors;
}
