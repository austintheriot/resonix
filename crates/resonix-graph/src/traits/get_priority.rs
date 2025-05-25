use crate::Priority;

pub trait GetPriority {
    fn get_priority(&self) -> Priority;
}
