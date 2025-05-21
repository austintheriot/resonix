use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;

use crate::{ResonixData, ResonixDataList};

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
#[derive(Debug, Clone, Default)]
pub struct ResonixDataResult {
    pub(crate) data_lists: Vec<ResonixDataList>,
}

impl ResonixDataResult {
    pub fn into_inner(self) -> Vec<ResonixDataList> {
        self.data_lists
    }

    pub fn from_vec<V: Into<Vec<ResonixDataList>>>(data_lists: V) -> Self {
        Self {
            data_lists: data_lists.into(),
        }
    }

    pub fn from_data_list<L: Into<ResonixDataList>>(data_list: L) -> Self {
        let data_list: ResonixDataList = data_list.into();
        let data_lists = vec![data_list];

        Self { data_lists }
    }

    pub fn from_value<D: Into<ResonixData>>(data: D) -> Self {
        let data = data.into();
        let data_list: Vec<ResonixData> = vec![data];
        let data_lists: Vec<ResonixDataList> = vec![ResonixDataList::from(data_list)];

        Self { data_lists }
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
