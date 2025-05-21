use core::any::Any;

use alloc::vec::Vec;

use crate::ResonixPortAddress;

pub trait ResonixParamNode: Any {
    fn as_any(&self) -> &dyn Any;

    fn input_port_addresses(&self) -> Vec<ResonixPortAddress>;

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress>;
}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
pub struct Param<T: ResonixParamNode + 'static>(pub T);

impl<P: ResonixParamNode + 'static> From<P> for Param<P> {
    fn from(value: P) -> Self {
        Param(value)
    }
}
