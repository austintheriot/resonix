use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;

use crate::ResonixDataList;

/// All the data lists that are returned when a node is run
///
/// For example
///      N Node
///    // \       <This level is the `ResonixDataResult`
///   A    B
///
/// ^ In this example, there are 2 data lists extending from Node N:
/// List A contains 2 data items.
/// List B contains 1 data item.
///
/// This structure enables multichannel audio.
#[derive(Debug, Clone)]
pub struct ResonixDataResult {
    pub(crate) data_lists: Vec<ResonixDataList>,
}

impl ResonixDataResult {
    pub fn into_inner(self) -> Vec<ResonixDataList> {
        self.data_lists
    }
}

impl Deref for ResonixDataResult {
    type Target = Vec<ResonixDataList>;

    fn deref(&self) -> &Self::Target {
        &self.data_lists
    }
}

impl DerefMut for ResonixDataResult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data_lists
    }
}

impl<F: Into<Vec<ResonixDataList>>> From<F> for ResonixDataResult {
    fn from(value: F) -> Self {
        let data_lists = value.into();
        Self { data_lists }
    }
}
