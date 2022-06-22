use super::current_status::CurrentStatus;

pub trait CurrentStatusAction {
    fn new(current_status: CurrentStatus) -> Self;

    fn get(&self) -> CurrentStatus;
    
    fn set(&mut self, status: CurrentStatus);
}
