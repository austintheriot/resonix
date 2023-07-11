use std::{
    any::Any,
    cell::{Ref, RefMut},
    fmt::Debug,
    sync::Arc, ops::{Deref, DerefMut},
};

use dyn_clone::DynClone;

use resonix_core::NumChannels;
use thiserror::Error;
#[cfg(feature = "dac")]
use {crate::UpdateNodeMessage, resonix_dac::DACConfig};

use crate::{Connection, NodeType, UpdateNodeError};

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

    fn num_input_connections(&self) -> usize;

    fn num_output_connections(&self) -> usize;

    fn num_incoming_channels(&self) -> NumChannels;

    fn num_outgoing_channels(&self) -> NumChannels;

    fn uid(&self) -> NodeUid;

    fn set_uid(&mut self, uid: NodeUid);

    fn name(&self) -> String;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    #[cfg(feature = "dac")]
    fn requires_audio_updates(&self) -> bool {
        false
    }

    #[cfg(feature = "dac")]
    fn update_from_dac_config(&mut self, _dac_config: Arc<DACConfig>) {}

    #[cfg(feature = "dac")]
    fn handle_update_node_message(
        &mut self,
        update_node_message: UpdateNodeMessage,
    ) -> Result<(), UpdateNodeError> {
        // by default, if a Node is not configured to receive `UpdateNodeMessage`
        // then an error will be returned
        Err(UpdateNodeError::NotConfigured {
            uid: update_node_message.node_uid,
        })
    }
}

dyn_clone::clone_trait_object!(Node);

#[derive(Debug, Clone)]
pub struct BoxedNode(pub Box<dyn Node>);

impl BoxedNode {
    pub fn new<N: Node + 'static>(node: N) -> Self {
        Self(Box::new(node))
    }
}

impl Deref for BoxedNode {
    type Target = Box<dyn Node>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoxedNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl Sync for BoxedNode {}

pub type NodeUid = u32;

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

    fn uid(&self) -> NodeUid {
        (**self).uid()
    }

    fn set_uid(&mut self, uid: NodeUid) {
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

    #[cfg(feature = "dac")]
    fn requires_audio_updates(&self) -> bool {
        (**self).requires_audio_updates()
    }

    #[cfg(feature = "dac")]
    fn update_from_dac_config(&mut self, dac_config: Arc<DACConfig>) {
        (**self).update_from_dac_config(dac_config)
    }

    #[cfg(feature = "dac")]
    fn handle_update_node_message(
        &mut self,
        update_node_message: UpdateNodeMessage,
    ) -> Result<(), UpdateNodeError> {
        (**self).handle_update_node_message(update_node_message)
    }
}
