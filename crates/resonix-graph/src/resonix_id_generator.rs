use crate::ResonixId;

pub trait ResonixIdGenerator {
    fn new_id(&mut self) -> ResonixId;
}
