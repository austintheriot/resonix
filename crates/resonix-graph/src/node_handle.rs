use crate::ResonixId;

pub struct ResonixNodeHandle(ResonixId);

impl ResonixNodeHandle {
    pub fn new(node_id: ResonixId) -> Self {
        Self(node_id)
    }
}

impl AsRef<ResonixId> for ResonixNodeHandle {
    fn as_ref(&self) -> &ResonixId {
        &self.0
    }
}
