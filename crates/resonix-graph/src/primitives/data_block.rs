use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;

use crate::primitives::Data;

pub struct DataBlock {
    vec: Vec<Data>,
}

impl Deref for DataBlock {
    type Target = Vec<Data>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl DerefMut for DataBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<V: Into<Vec<Data>>> From<V> for DataBlock {
    fn from(value: V) -> Self {
        Self { vec: value.into() }
    }
}

impl DataBlock {
    pub fn into_inner(self) -> Vec<Data> {
        self.vec
    }
}
