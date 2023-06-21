use std::{
    cell::{Ref, RefMut},
    fmt::Debug,
};

use uuid::Uuid;
use dyn_clone::DynClone;

use crate::{Connection, NodeType};

pub trait Node
where
    Self: Debug + DynClone,
{
    fn process(&mut self, inputs: &[Ref<Connection>], outputs: &mut [RefMut<Connection>]);

    fn node_type(&self) -> NodeType;

    fn num_inputs(&self) -> usize;

    fn num_outputs(&self) -> usize;

    fn uuid(&self) -> &Uuid;

    fn name(&self) -> String;
}

dyn_clone::clone_trait_object!(Node);

pub type BoxedNode = Box<dyn Node>;
