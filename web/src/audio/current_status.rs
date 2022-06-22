#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum CurrentStatus {
    PLAY,
    PAUSE,
}

impl From<bool> for CurrentStatus {
    fn from(is_playing: bool) -> Self {
        match is_playing {
            true => CurrentStatus::PLAY,
            false => CurrentStatus::PAUSE,
        }
    }
}

impl From<CurrentStatus> for bool {
    fn from(current_status: CurrentStatus) -> Self {
        let is_playing = match current_status {
            CurrentStatus::PLAY => true,
            CurrentStatus::PAUSE => false,
        };
        is_playing
    }
}
