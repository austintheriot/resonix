#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PlayStatus {
    Play,
    Pause,
}

impl From<bool> for PlayStatus {
    fn from(is_playing: bool) -> Self {
        match is_playing {
            true => PlayStatus::Play,
            false => PlayStatus::Pause,
        }
    }
}

impl From<PlayStatus> for bool {
    fn from(play_status: PlayStatus) -> Self {
        match play_status {
            PlayStatus::Play => true,
            PlayStatus::Pause => false,
        }
    }
}
