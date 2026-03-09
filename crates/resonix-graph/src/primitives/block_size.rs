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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_deref() {
        let block_size = BlockSize::new(512);
        assert_eq!(*block_size, 512);
    }

    #[test]
    fn max_block_size_is_65536() {
        assert_eq!(BlockSize::MAX_BLOCK_SIZE, 65536);
    }

    #[test]
    fn from_usize_creates_block_size_with_that_value() {
        let block_size = BlockSize::from(1024usize);
        assert_eq!(*block_size, 1024);
    }

    #[test]
    fn from_i32_casts_to_usize() {
        let block_size = BlockSize::from(256i32);
        assert_eq!(*block_size, 256);
    }

    #[test]
    fn block_sizes_with_same_value_are_equal() {
        assert_eq!(BlockSize::new(128), BlockSize::new(128));
    }

    #[test]
    fn smaller_block_size_is_less_than_larger() {
        assert!(BlockSize::new(64) < BlockSize::new(128));
    }
}
