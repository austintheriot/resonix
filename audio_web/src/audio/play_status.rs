#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PlayStatus {
    PLAY,
    PAUSE,
}

impl From<bool> for PlayStatus {
    fn from(is_playing: bool) -> Self {
        match is_playing {
            true => PlayStatus::PLAY,
            false => PlayStatus::PAUSE,
        }
    }
}

impl From<PlayStatus> for bool {
    fn from(play_status: PlayStatus) -> Self {
        match play_status {
            PlayStatus::PLAY => true,
            PlayStatus::PAUSE => false,
        }
    }
}
