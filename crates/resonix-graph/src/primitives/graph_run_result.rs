use hashbrown::HashMap;

use crate::primitives::{Data, Id};

pub struct GraphRunResult {
    outputs: HashMap<Id, Data>,
}

impl GraphRunResult {
    pub fn new(outputs: HashMap<Id, Data>) -> Self {
        Self { outputs }
    }

    pub fn outputs(&self) -> &HashMap<Id, Data> {
        &self.outputs
    }
}
