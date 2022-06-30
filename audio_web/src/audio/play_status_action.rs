use super::play_status::PlayStatus;

pub trait PlayStatusAction {
    fn new(play_status: PlayStatus) -> Self;

    fn get(&self) -> PlayStatus;

    fn set(&mut self, status: PlayStatus);
}
