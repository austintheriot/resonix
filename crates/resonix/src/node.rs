use std::fmt::Debug;

use dyn_clone::DynClone;
use uuid::Uuid;

use crate::{Connection, NodeType};

pub trait Node
where
    Self: Debug + DynClone + Send + Sync,
{
    fn process(&mut self, inputs: &[Connection], outputs: &mut [Connection]);

    fn node_type(&self) -> NodeType;

    fn num_inputs(&self) -> usize;

    fn num_outputs(&self) -> usize;

    fn uuid(&self) -> &Uuid;

    fn name(&self) -> String;
}

dyn_clone::clone_trait_object!(Node);

pub type BoxedNode = Box<dyn Node>;
