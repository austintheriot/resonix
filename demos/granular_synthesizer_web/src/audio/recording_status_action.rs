use super::recording_status::RecordingStatus;

pub trait RecordingStatusAction {
    fn new(play_status: RecordingStatus) -> Self;

    fn get(&self) -> RecordingStatus;

    fn set(&mut self, status: RecordingStatus);
}
