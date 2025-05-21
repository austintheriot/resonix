use alloc::vec::Vec;

use crate::{
    GenerateId, ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult, ResonixId,
    ResonixPortAddress, ResonixPortAddressDirection,
};

pub struct ConstantNode {
    node_id: ResonixId,
}

impl ConstantNode {
    pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(0usize);

    pub fn new<G: GenerateId>(id_generator: &mut G) -> Self {
        let node_id = id_generator.generate_id();
        Self { node_id }
    }

    pub fn output_port_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::OUTPUT_PORT_ID,
            ResonixPortAddressDirection::Output,
        )
    }
}

impl ResonixAudioNode for ConstantNode {
    fn next(&mut self) -> ResonixDataResult {
        let connection_data: Vec<ResonixDataList> =
            vec![ResonixDataList::from([ResonixData::F32(1.0)])];

        connection_data.into()
    }

    fn assign_inputs(&mut self, _inputs: ResonixDataResult) {
        // it takes no inputs
    }

    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![]
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.output_port_address()]
    }

    fn node_id(&self) -> ResonixId {
        self.node_id
    }
}
