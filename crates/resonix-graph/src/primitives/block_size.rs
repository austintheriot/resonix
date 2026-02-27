use core::ops::Deref;

use nohash_hasher::IsEnabled;

#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockSize(usize);

impl BlockSize {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    // TODO: should this number be larger? = 2^16
    pub const MAX_BLOCK_SIZE: usize = 65536;
}

impl IsEnabled for BlockSize {}

// other convenience implementations possible here

impl Deref for BlockSize {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for BlockSize {
    fn from(value: i32) -> Self {
        BlockSize(value as usize)
    }
}

impl From<usize> for BlockSize {
    fn from(value: usize) -> Self {
        BlockSize(value)
    }
}
