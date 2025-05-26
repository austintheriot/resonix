use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;

use crate::primitives::Data;

#[derive(Debug, Clone)]
pub struct DataList(pub(crate) Vec<Data>);

impl DataList {
    pub fn new(resonix_data: Vec<Data>) -> Self {
        DataList(resonix_data)
    }

    pub fn empty() -> Self {
        DataList(Vec::new())
    }

    pub fn into_inner(self) -> Vec<Data> {
        self.0
    }
}

impl Deref for DataList {
    type Target = Vec<Data>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<F: Into<Vec<Data>>> From<F> for DataList {
    fn from(value: F) -> Self {
        Self(value.into())
    }
}
