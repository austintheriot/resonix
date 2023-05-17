use crate::Index;

pub struct IntMap<V>(Vec<Option<V>>)
where
    V: Index;

impl<V> IntMap<V>
where
    V: Index,
{
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Inserts the element at the index returned by its `IntMap` trait
    /// If there is not enough room in the Vector, the Vector is resized to fit
    pub fn insert(&mut self, v: V) -> Option<V> {
        let i: usize = v.id();
        if i >= self.0.len() {
            self.0.resize_with(i + 1, || None);
        }
        std::mem::replace(&mut self.0[i], Some(v))
    }

    pub fn get(&self, i: impl Index) -> Option<&V> {
        self.0.get(i.id()).unwrap_or(&None).as_ref()
    }
}

#[cfg(test)]
mod test_vec_map_struct {
    use crate::{Index, IntMap};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
    struct Element(usize);

    impl Index for Element {
        fn id(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn it_stores_an_element_and_returns_it() {
        let mut index_map = IntMap::new();
        let element = Element(15);
        index_map.insert(element);

        assert_eq!(index_map.get(15), Some(&element));
    }

    #[test]
    fn it_stores_multiple_elements_and_returns_them() {
        let mut index_map = IntMap::new();
        let element1 = Element(0);
        let element2 = Element(15);
        let element3 = Element(119);
        index_map.insert(element1);
        index_map.insert(element2);
        index_map.insert(element3);

        assert_eq!(index_map.get(0), Some(&element1));
        assert_eq!(index_map.get(15), Some(&element2));
        assert_eq!(index_map.get(119), Some(&element3));
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

        let mut index_map = IntMap::new();
        let element1 = ElementWithOtherData(0, 0);
        let element2 = ElementWithOtherData(0, 1);
        let element3 = ElementWithOtherData(1, 1);
        let element4 = ElementWithOtherData(1, 2);
        index_map.insert(element1);
        index_map.insert(element2);
        index_map.insert(element3);
        index_map.insert(element4);

        assert_eq!(index_map.get(0), Some(&element2));
        assert_eq!(index_map.get(1), Some(&element4));
    }
}
