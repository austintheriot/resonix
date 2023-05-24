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

    // returns the number of items in the set
    // (i.e. only the entries that are currently set to `Some<V>`)
    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.indexes.iter().map(|i| *self.get(*i).as_ref().unwrap())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.indexes.iter().map(|i| {
            // Uses indexes to get mutable references to underlying data
            //
            //  This is safe because:
            //  -   index is guaranteed to be within bounds
            //  -   indexes are guaranteed not to contain duplicates
            unsafe { &mut *self.data.as_mut_ptr().offset(*i as isize) }
                .as_mut()
                .unwrap()
        })
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

    pub fn extend<I: IntoIterator<Item = V> + Clone>(&mut self, iter: I) {
        // kind of hacky ¯\_(ツ)_/¯ but keeps track of both index and adding data to
        // vector without cloning the iterator
        let iter = iter.into_iter();
        for el in iter {
            self.insert(el);
        }
    }

    /// Removes any elements (beginning with largest indexes first).
    /// Returns the number of elements removed
    pub fn truncate(&mut self, len: usize) -> usize {
        let mut elements_removed = 0;
        let prev_len = self.len();

        let elements_to_remove = if len >= prev_len {
            return elements_removed;
        } else {
            prev_len - len
        };

        let indexes_to_remove: Vec<_> = self
            .indexes
            .iter()
            .rev()
            .take(elements_to_remove)
            .map(ToOwned::to_owned)
            .collect();

        indexes_to_remove.iter().for_each(|i| {
            elements_removed += 1;
            self.remove(*i);
        });

        elements_removed
    }

    pub fn get(&self, i: impl Index) -> Option<&V> {
        self.data.get(i.id()).unwrap_or(&None).as_ref()
    }

    pub fn get_mut(&mut self, i: impl Index) -> Option<&mut V> {
        self.data.get_mut(i.id()).and_then(|v| v.as_mut())
    }

    pub fn first(&self) -> Option<&V> {
        self.data.iter().next().and_then(|v| v.as_ref())
    }

    pub fn first_mut(&mut self) -> Option<&mut V> {
        self.data.iter_mut().next().and_then(|v| v.as_mut())
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
                std::mem::take(&mut self.data[i])
            } else {
                None
            }
        })
    }

    /// Removes and returns the element with the smallest index
    pub fn pop_first(&mut self) -> Option<V> {
        self.indexes.pop_first().and_then(|i| {
            if i < self.data.len() {
                std::mem::take(&mut self.data[i])
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
        assert_eq!(int_set.len(), 1)
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
        assert_eq!(int_set.len(), 3);
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
        assert_eq!(int_set.len(), 2);
    }

    #[test]
    fn it_should_allow_removing_entry() {
        let mut int_set = IntSet::new();
        let element1 = Element(0);
        int_set.insert(element1);
        int_set.remove(0);
        assert_eq!(int_set.get(0), None);
        assert_eq!(int_set.len(), 0);
    }

    #[test]
    fn it_should_not_throw_when_trying_to_remove_nonexistent_entry() {
        let mut int_set: IntSet<Element> = IntSet::new();
        int_set.remove(0);
        assert_eq!(int_set.get(0), None);
        assert_eq!(int_set.len(), 0);
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
        assert_eq!(int_set.len(), 1);
    }

    #[test]
    fn it_should_allow_extending() {
        let mut int_set = IntSet::new();
        let new_elements = (0..=5).map(|i| Element(i));
        int_set.extend(new_elements);
        assert_eq!(int_set.get(0), Some(&Element(0)));
        assert_eq!(int_set.get(5), Some(&Element(5)));
        assert_eq!(int_set.get(6), None);
        assert_eq!(int_set.len(), 6);
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

        assert_eq!(int_set.len(), 3);
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

        let mut iterator = int_set.iter();
        assert_eq!(iterator.next(), Some(&element1));
        assert_eq!(iterator.next(), Some(&element2));
        assert_eq!(iterator.next(), Some(&element3));

        assert_eq!(int_set.len(), 3);
    }

    #[test]
    fn it_should_be_able_to_iterate_over_mutable_reference() {
        let mut int_set = IntSet::new();
        let mut element1 = Element(0);
        let mut element2 = Element(15);
        let mut element3 = Element(119);

        int_set.insert(element1);
        int_set.insert(element2);
        int_set.insert(element3);

        let mut iterator = int_set.iter_mut();
        assert_eq!(iterator.next(), Some(&mut element1));
        assert_eq!(iterator.next(), Some(&mut element2));
        assert_eq!(iterator.next(), Some(&mut element3));
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

            assert_eq!(int_set.pop_last(), Some(Element(119)));
            assert_eq!(int_set.len(), 2)
        }

        #[test]
        fn extending() {
            let mut int_set = IntSet::new();
            let new_elements = (50..=100).map(|i| Element(i));
            int_set.extend(new_elements);
            assert_eq!(int_set.pop_last(), Some(Element(100)));
            assert_eq!(int_set.len(), 50);
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
            let elements_removed = int_set.truncate(1);

            assert_eq!(int_set.pop_last(), Some(element1));
            assert_eq!(elements_removed, 2);
            assert_eq!(int_set.len(), 0);
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

        assert_eq!(int_set.pop_first(), Some(Element(0)));
        assert_eq!(int_set.len(), 2);
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
        assert_eq!(int_set.len(), 3);
    }
}
