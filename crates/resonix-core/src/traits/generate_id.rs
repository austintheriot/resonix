use crate::primitives::Id;

pub trait GenerateId {
    fn generate_id(&mut self) -> Id;
}
