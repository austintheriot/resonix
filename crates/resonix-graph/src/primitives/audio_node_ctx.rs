use crate::primitives::{BlockSize, CurrentTime};

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct AudioNodeCtx {
    pub current_time: CurrentTime,
    pub block_size: BlockSize,
}
