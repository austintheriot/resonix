use crate::primitives::ResonixId;

pub trait GenerateId {
    fn generate_id(&mut self) -> ResonixId;
}
