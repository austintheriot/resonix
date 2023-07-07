use std::{
    any::Any,
    cell::{Ref, RefMut},
    fmt::Debug,
};

use dyn_clone::DynClone;

use resonix_core::NumChannels;
use thiserror::Error;

use crate::{Connection, NodeType};

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddConnectionError {
    #[error("Could not add incoming connection for {name:?}, because it only accepts outgoing connections")]
    CantAcceptInputConnections { name: String },
    #[error("Could not add outgoing connection for {name:?}, because it only accepts incoming connections")]
    CantAcceptOutputConnections { name: String },
}

pub trait Node
where
    Self: Debug + Send + DynClone,
{
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    );

    fn node_type(&self) -> NodeType;

    fn num_input_connections(&self) -> usize;

    fn num_output_connections(&self) -> usize;

    fn num_incoming_channels(&self) -> NumChannels;

    fn num_outgoing_channels(&self) -> NumChannels;

    fn uid(&self) -> u32;

    fn set_uid(&mut self, uid: u32);

    fn name(&self) -> String;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

dyn_clone::clone_trait_object!(Node);

pub type BoxedNode = Box<dyn Node>;

impl Node for BoxedNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        (**self).process(inputs, outputs)
    }

    fn node_type(&self) -> NodeType {
        (**self).node_type()
    }

    fn num_input_connections(&self) -> usize {
        (**self).num_input_connections()
    }

    fn num_output_connections(&self) -> usize {
        (**self).num_output_connections()
    }

    fn num_incoming_channels(&self) -> NumChannels {
        (**self).num_incoming_channels()
    }

    fn num_outgoing_channels(&self) -> NumChannels {
        (**self).num_outgoing_channels()
    }

    fn uid(&self) -> u32 {
        (**self).uid()
    }

    fn set_uid(&mut self, uid: u32) {
        (**self).set_uid(uid)
    }

    fn name(&self) -> String {
        (**self).name()
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        (**self).as_any_mut()
    }
}
