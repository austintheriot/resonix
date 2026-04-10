use bon::Builder;

use crate::primitives::{BlockSize, CurrentTime, SampleRate};

#[non_exhaustive]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Builder)]
pub struct AudioNodeCtx {
    pub current_time: CurrentTime,
    pub block_size: BlockSize,
    pub sample_rate: SampleRate,
}
