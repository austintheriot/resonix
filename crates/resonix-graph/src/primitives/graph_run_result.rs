use hashbrown::HashMap;

use crate::primitives::{Data, PortAddress};

pub struct GraphRunResult {
    outputs: HashMap<PortAddress, Data>,
}

impl GraphRunResult {
    pub fn new(outputs: HashMap<PortAddress, Data>) -> Self {
        Self { outputs }
    }

    pub fn outputs(&self) -> &HashMap<PortAddress, Data> {
        &self.outputs
    }
}
