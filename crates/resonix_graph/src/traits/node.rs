use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;
use uuid::Uuid;

use crate::{Connection, NodeType};

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
}

dyn_clone::clone_trait_object!(Node);

pub type BoxedNode = Box<dyn Node>;
