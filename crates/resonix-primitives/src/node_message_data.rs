pub struct NodeMessageData {}

impl<
        A: resonix_types::Amplitude,
        S: resonix_types::Sample<A>,
        F: resonix_types::Frame<A, S>,
        U: resonix_types::AudioContextUid,
    > resonix_types::NodeMessageData<A, S, F, U> for NodeMessageData
{
    fn node_uid(&self) -> &U {
        todo!()
    }

    fn message_data(&self) -> &resonix_types::MessageData {
        todo!()
    }

    fn into_message_data(self) -> resonix_types::MessageData {
        todo!()
    }
}
