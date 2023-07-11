use std::marker::PhantomData;

use crate::{
    messages::{MessageError, UpdateNodeError, UpdateNodeMessage},
    AudioContext, AudioInit, AudioUninit, Node, NodeUid,
};

#[derive(thiserror::Error, Debug)]
pub enum NodeHandleMessageError {
    #[error("A message was sent from the `NodeHandle` to the processor, but no corresponding message was received")]
    NoMatchingMessageReceived,
    #[error("Error occurred while communicating with Processor. Original error: {0:?}")]
    MessageError(#[from] MessageError),
}

/// The `NodeHandle` allows mutating audio a node's data from
/// the main thread, even after that node has been sent to
/// the audio thread. `NodeHandle` implements specific functionality
/// for whatever generic `node_type` the `NodeHandle` is.
///
/// This is accomplished by sending messages between the main
/// thread and the audio thread.
///
/// All audio graph mutations are processed in the order in which
/// they were received.
///
/// This struct can be safely and cheaply cloned
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeHandle<NodeType: Node> {
    pub(crate) uid: NodeUid,
    pub(crate) node_type: PhantomData<NodeType>,
}

impl<N: Node> NodeHandle<N> {
    /// Synchronously updates a Node with the given message.
    ///
    /// Note: it is up to the caller to make sure that
    /// the message conforms to the a type that the receiving
    /// Node is configured to receive.
    ///
    /// If a message a data of unexpected type is received,
    /// the processor will return an error back to the caller.
    #[cfg(feature = "dac")]
    pub fn update_sync<D: Send + 'static>(
        &self,
        audio_context: &mut AudioContext<AudioUninit>,
        message_data: D,
    ) -> Result<(), UpdateNodeError> {
        audio_context.handle_update_node_message(UpdateNodeMessage {
            node_uid: self.uid,
            data: Box::new(message_data),
        })
    }

    /// Asynchronously sends a message to the audio thread,
    /// where the node is updated. Async function returns
    /// once the result of the update is returned from the
    /// audio thread.
    ///
    /// Note: it is up to the caller to make sure that
    /// the message conforms to the a type that the receiving
    /// Node is configured to receive.
    ///
    /// If a message a data of unexpected type is received,
    /// the audio thread will return an error back to the caller.
    #[cfg(feature = "dac")]
    pub async fn update_async<D: Send + 'static>(
        &self,
        audio_context: &mut AudioContext<AudioInit>,
        message_data: D,
    ) -> Result<(), MessageError> {
        audio_context
            .handle_update_node_message(UpdateNodeMessage {
                node_uid: self.uid,
                data: Box::new(message_data),
            })
            .await
    }
}

impl<N: Node> Clone for NodeHandle<N> {
    fn clone(&self) -> Self {
        Self {
            uid: self.uid,
            node_type: PhantomData,
        }
    }
}

impl<N: Node> Copy for NodeHandle<N> {}

impl<N: Node> AsRef<NodeUid> for NodeHandle<N> {
    fn as_ref(&self) -> &NodeUid {
        &self.uid
    }
}
