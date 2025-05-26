/// Indicates which direction a port can give/receive information
///
/// If Input, the port accepts data at that location, if Output,
/// the Node sends out information at that location.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortAddressDirection {
    Input,
    Output,
}
