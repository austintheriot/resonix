use std::{
    collections::BTreeSet,
    slice::{Iter, IterMut},
    vec::IntoIter,
};

use crate::Index;

/// A set of values, indexed by a `usize` integer.
///
/// This structure is best used with densely-packed, small-integer indexes.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntSet<V: Index> {
    data: Vec<Option<V>>,
    indexes: BTreeSet<usize>,
}

/// Implements a wrapper around the raw Vec iterator that skips all `None` values stored in the Vec
pub struct IntSetIterator<V>(IntoIter<Option<V>>);

impl<V> Iterator for IntSetIterator<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        for option in self.0.by_ref() {
            if let Some(value) = option {
                return Some(value);
            }
        }
        None
    }
}

impl<V: Index> IntoIterator for IntSet<V> {
    type Item = V;
    type IntoIter = IntSetIterator<V>;

    fn into_iter(self) -> Self::IntoIter {
        IntSetIterator(self.data.into_iter())
    }
}

/// Implements a wrapper around the raw Vec iterator that skips all `None` values stored in the Vec
pub struct IntSetIteratorRef<'a, V>(Iter<'a, Option<V>>);

impl<'a, V> Iterator for IntSetIteratorRef<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        for option in self.0.by_ref() {
            if let Some(value) = option {
                return Some(value);
            }
        }
        None
    }
}

impl<'a, V: Index> IntoIterator for &'a IntSet<V> {
    type Item = &'a V;
    type IntoIter = IntSetIteratorRef<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntSetIteratorRef(self.data.iter())
    }
}

/// Implements a mutable wrapper around the raw Vec iterator that skips all `None` values stored in the Vec
pub struct IntSetIteratorMut<'a, V>(IterMut<'a, Option<V>>);

impl<'a, V> Iterator for IntSetIteratorMut<'a, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        for option in self.0.by_ref() {
            if let Some(value) = option {
                return Some(value);
            }
        }
        None
    }
}

impl<'a, V: Index> IntoIterator for &'a mut IntSet<V> {
    type Item = &'a mut V;
    type IntoIter = IntSetIteratorMut<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntSetIteratorMut(self.data.iter_mut())
    }
}

impl<V> IntSet<V>
where
    V: Index,
{
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            indexes: BTreeSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.iter().filter(|el| el.is_some()).count()
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.data.iter().filter_map(|el| el.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().filter_map(|el| el.as_mut())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            indexes: BTreeSet::new(),
        }
    }

    /// Inserts the element at the index returned by its `IntSet` trait
    /// If there is not enough room in the underlying Vec, it is resized to fit.
    /// If an element existed at this location previously, the previous element is returned.
    pub fn insert(&mut self, v: V) -> Option<V> {
        let i: usize = v.id();
        if i >= self.data.len() {
            self.data.resize_with(i + 1, || None);
        }
        self.indexes.insert(i);
        std::mem::replace(&mut self.data[i], Some(v))
    }

    pub fn extend<I: IntoIterator<Item = Option<V>> + Clone>(&mut self, iter: I) {
        // kind of hacky ¯\_(ツ)_/¯
        let iter = iter.into_iter();
        for el in iter {
            if let Some(el) = el {
                self.insert(el);
            }
        }
    }

    pub fn truncate(&mut self, len: usize) {
        self.data.drain((len + 1)..).flatten().for_each(|el| {
            self.indexes.remove(&el.id());
        })
    }

    pub fn get(&self, i: impl Index) -> Option<&V> {
        self.data.get(i.id()).unwrap_or(&None).as_ref()
    }

    pub fn first(&self) -> Option<&V> {
        self.data
            .iter()
            .find(|v| v.is_some())
            .and_then(|v| v.as_ref())
    }

    pub fn first_mut(&mut self) -> Option<&mut V> {
        self.data
            .iter_mut()
            .find(|v| v.is_some())
            .and_then(|v| v.as_mut())
    }

    pub fn remove(&mut self, i: impl Index) -> Option<V> {
        let i: usize = i.id();
        if i < self.data.len() {
            self.indexes.remove(&i);
            std::mem::take(&mut self.data[i])
        } else {
            None
        }
    }

    /// Removes and returns the element with the largest index
    pub fn pop_last(&mut self) -> Option<V> {
        self.indexes.pop_last().and_then(|i| {
            if i < self.data.len() {
                self.data.remove(i)
            } else {
                None
            }
        })
    }

    /// Removes and returns the element with the smallest index
    pub fn pop_first(&mut self) -> Option<V> {
        self.indexes.pop_first().and_then(|i| {
            if i < self.data.len() {
                self.data.remove(i)
            } else {
                None
            }
        })
    }

    pub fn contains(&self, i: impl Index) -> bool {
        self.data
            .get(i.id())
            .map(Option::is_some)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod test_vec_map_struct {
    use crate::{Index, IntSet};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
    struct Element(usize);

    impl Index for Element {
        fn id(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn it_stores_an_element_and_returns_it() {
        let mut int_set = IntSet::new();
        let element = Element(15);
        int_set.insert(element);

        assert_eq!(int_set.get(15), Some(&element));
    }

    #[test]
    fn it_stores_multiple_elements_and_returns_them() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);
        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);

        assert_eq!(int_set.get(0), Some(&element1));
        assert_eq!(int_set.get(15), Some(&element2));
        assert_eq!(int_set.get(119), Some(&element3));
    }

    #[test]
    fn it_overwrites_values_when_the_id_is_the_same() {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
        struct ElementWithOtherData(usize, i32);

        impl Index for ElementWithOtherData {
            fn id(&self) -> usize {
                self.0
            }
        }

        let mut int_set = IntSet::new();
        let element1 = ElementWithOtherData(0, 0);
        let element2 = ElementWithOtherData(0, 1);
        let element3 = ElementWithOtherData(1, 1);
        let element4 = ElementWithOtherData(1, 2);
        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);
        int_set.insert(element4);

        assert_eq!(int_set.get(0), Some(&element2));
        assert_eq!(int_set.get(1), Some(&element4));
    }

    #[test]
    fn it_should_allow_removing_entry() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        int_set.insert(element1);
        int_set.remove(0);
        assert_eq!(int_set.get(0), None);
    }

    #[test]
    fn it_should_not_throw_when_trying_to_remove_nonexistent_entry() {
        let mut int_set: IntSet<Element> = IntSet::new();
        int_set.remove(0);
        assert_eq!(int_set.get(0), None);
    }

    #[test]
    fn it_should_allow_removing_items_in_bulk_via_truncate() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);
        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);

        assert_eq!(int_set.get(0), Some(&element1));
        assert_eq!(int_set.get(15), Some(&element2));
        assert_eq!(int_set.get(119), Some(&element3));

        int_set.truncate(1);

        assert_eq!(int_set.get(0), Some(&element1));
        assert_eq!(int_set.get(15), None);
        assert_eq!(int_set.get(119), None);
    }

    #[test]
    fn it_should_allow_extending() {
        let mut int_set = IntSet::new();
        let new_elements = (0..=5).map(|i| Some(Element(i)));
        int_set.extend(new_elements);
        assert_eq!(int_set.get(0), Some(&Element(0)));
        assert_eq!(int_set.get(5), Some(&Element(5)));
        assert_eq!(int_set.get(6), None);
    }

    #[test]
    fn it_should_allow_counting_number_of_elements() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);
        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);

        assert_eq!(int_set.len(), 3)
    }

    #[test]
    fn it_should_be_able_to_iterate_over_immutable_reference() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);

        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);

        let mut examples = vec![];

        for el in &int_set {
            examples.push(el);
        }

        assert_eq!(examples.len(), 3)
    }

    #[test]
    fn it_should_be_able_to_iterate_over_mutable_reference() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);

        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);

        let mut examples = vec![];

        for el in &mut int_set {
            examples.push(el);
        }

        assert_eq!(examples.len(), 3)
    }

    #[test]
    fn it_should_be_able_to_produce_owned_iterator() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);

        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);

        let collected_vec: Vec<_> = int_set.into_iter().map(|el| el.0).collect();

        assert_eq!(collected_vec.len(), 3)
    }

    #[cfg(test)]
    mod it_should_allow_popping_largest_value {
        use super::Element;
        use crate::IntSet;

        #[test]
        fn insert() {
            let mut int_set = IntSet::new();
            let element1 = Element(0);
            let element2 = Element(15);
            let element3 = Element(119);

            int_set.insert(element3);
            int_set.insert(element2);
            int_set.insert(element1);

            assert_eq!(int_set.pop_last(), Some(Element(119)))
        }

        #[test]
        fn extending() {
            let mut int_set = IntSet::new();
            let new_elements = (50..=100).map(|i| Some(Element(i)));
            int_set.extend(new_elements);
            assert_eq!(int_set.pop_last(), Some(Element(100)))
        }

        #[test]
        fn truncate() {
            let mut int_set = IntSet::new();
            let element1 = Element(0);
            let element2 = Element(15);
            let element3 = Element(119);
            int_set.insert(element1);
            int_set.insert(element2);
            int_set.insert(element3);
            int_set.truncate(1);

            assert_eq!(int_set.pop_last(), Some(element1));
        }
    }

    #[test]
    fn it_should_allow_popping_smallest_value() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);

        int_set.insert(element3);
        int_set.insert(element2);
        int_set.insert(element1);

        assert_eq!(int_set.pop_first(), Some(Element(0)))
    }

    #[test]
    fn it_should_allow_querying_whether_index_is_contained() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);

        int_set.insert(element3);
        int_set.insert(element2);
        int_set.insert(element1);

        assert!(int_set.contains(0));
        assert!(!int_set.contains(14));
        assert!(!int_set.contains(1000));
    }
}
