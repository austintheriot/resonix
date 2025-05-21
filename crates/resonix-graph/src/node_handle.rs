use crate::ResonixId;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixNodeHandle(ResonixId);

impl ResonixNodeHandle {
    pub fn new(node_id: ResonixId) -> Self {
        Self(node_id)
    }

    pub fn node_id(&self) -> ResonixId {
        self.0
    }
}
