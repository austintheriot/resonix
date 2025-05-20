use crate::ResonixId;

pub struct ResonixNodeHandle(ResonixId);

impl ResonixNodeHandle {
    pub fn new(node_id: ResonixId) -> Self {
        Self(node_id)
    }
}
