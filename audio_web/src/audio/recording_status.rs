#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RecordingStatus {
    Recording,
    Stop,
}

impl From<bool> for RecordingStatus {
    fn from(is_recording: bool) -> Self {
        match is_recording {
            true => RecordingStatus::Recording,
            false => RecordingStatus::Stop,
        }
    }
}

impl From<RecordingStatus> for bool {
    fn from(play_status: RecordingStatus) -> Self {
        match play_status {
            RecordingStatus::Recording => true,
            RecordingStatus::Stop => false,
        }
    }
}
