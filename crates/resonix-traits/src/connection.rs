use crate::ConnectionDescriptor;

pub trait Connection {
    fn descriptor(&self) -> &impl ConnectionDescriptor;
}
