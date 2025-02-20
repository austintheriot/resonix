use crate::{AudioContextUid, MessageData};

pub trait NodeMessageData {
    fn node_uid(&self) -> impl AudioContextUid;

    fn message_data(&self) -> &MessageData;

    fn into_message_data(self) -> MessageData;
}
