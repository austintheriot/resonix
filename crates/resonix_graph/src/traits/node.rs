use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;
use petgraph::prelude::EdgeIndex;
use thiserror::Error;
use uuid::Uuid;

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
        inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    );

    fn node_type(&self) -> NodeType;

    fn num_inputs(&self) -> usize;

    fn num_outputs(&self) -> usize;

    fn uuid(&self) -> &Uuid;

    fn name(&self) -> String;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn incoming_connection_indexes(&self) -> &[EdgeIndex];

    fn outgoing_connection_indexes(&self) -> &[EdgeIndex];

    fn add_incoming_connection_index(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError>;

    fn add_outgoing_connection_index(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError>;
}

dyn_clone::clone_trait_object!(Node);

pub type BoxedNode = Box<dyn Node>;

impl Node for BoxedNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        (**self).process(inputs, outputs)
    }

    fn node_type(&self) -> NodeType {
        (**self).node_type()
    }

    fn num_inputs(&self) -> usize {
        (**self).num_inputs()
    }

    fn num_outputs(&self) -> usize {
        (**self).num_outputs()
    }

    fn uuid(&self) -> &Uuid {
        (**self).uuid()
    }

    fn name(&self) -> String {
        (**self).name()
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    #[inline]
    fn incoming_connection_indexes(&self) -> &[EdgeIndex] {
        (**self).incoming_connection_indexes()
    }

    #[inline]
    fn outgoing_connection_indexes(&self) -> &[EdgeIndex] {
        (**self).outgoing_connection_indexes()
    }

    fn add_incoming_connection_index(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError> {
        (**self).add_incoming_connection_index(edge_index)
    }

    fn add_outgoing_connection_index(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError> {
        (**self).add_outgoing_connection_index(edge_index)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        (**self).as_any_mut()
    }
}
