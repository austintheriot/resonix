use crate::{Amplitude, AudioContextUid, Frame, MessageData, Sample};

pub trait NodeMessageData<A: Amplitude, S: Sample<A>, F: Frame<A, S>, U: AudioContextUid> {
    fn node_uid(&self) -> &U;

    fn message_data(&self) -> &MessageData;

    fn into_message_data(self) -> MessageData;
}
