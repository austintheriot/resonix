use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;

use crate::ResonixData;

#[derive(Debug, Clone)]
pub struct ResonixDataList(Vec<ResonixData>);

impl ResonixDataList {
    pub fn into_inner(self) -> Vec<ResonixData> {
        self.0
    }
}

impl Deref for ResonixDataList {
    type Target = Vec<ResonixData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ResonixDataList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<F: Into<Vec<ResonixData>>> From<F> for ResonixDataList {
    fn from(value: F) -> Self {
        Self(value.into())
    }
}
