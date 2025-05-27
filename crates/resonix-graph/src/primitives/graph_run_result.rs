use hashbrown::HashMap;

use crate::primitives::{Data, Id};

pub struct GraphRunResult {
    outputs: HashMap<Id, Data>,
}

impl GraphRunResult {
    pub fn outputs(&self) -> &HashMap<Id, Data> {
        &self.outputs
    }
}
