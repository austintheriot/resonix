pub struct NodeMessageData {}

impl resonix_types::NodeMessageData for NodeMessageData {
    fn node_uid(&self) -> impl resonix_types::AudioContextUid {
        todo!()
    }

    fn message_data(&self) -> &resonix_types::MessageData {
        todo!()
    }

    fn into_message_data(self) -> resonix_types::MessageData {
        todo!()
    }
}
