/// Indicates which direction a port can give/receive information
///
/// If Input, the port accepts data at that location, if Output,
/// the Node sends out information at that location.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortAddressDirection {
    /// Node accepts data from other Nodes at this port location
    Input,
    /// Node emits data to other Nodes at this port loation
    Output,
    /// Node emits data to the external system at this port location
    ExternalOutput,
    /// Node accepts data from the external system at this port location
    ExternalInput,
}
