use std::hash::{Hash, Hasher};

#[derive(Debug, Default, Clone)]
pub struct Connection {
    /// where the connection is coming from
    from_index: usize,
    /// where the connection is going to
    to_index: usize,
    /// the data that the connection is carrying (if any)
    data: f32,
    uid: u32,
}

impl Connection {
    pub fn new() -> Self {
        Self::from_indexes(0, 0)
    }

    pub fn from_indexes(from_index: usize, to_index: usize) -> Self {
        Self {
            data: 0.0,
            from_index,
            to_index,
            uid: 0,
        }
    }

    pub fn from_index(&self) -> usize {
        self.from_index
    }

    pub fn to_index(&self) -> usize {
        self.to_index
    }

    pub fn data(&self) -> f32 {
        self.data
    }

    pub fn set_data(&mut self, data: f32) -> &mut Self {
        self.data = data;
        self
    }

    pub fn uid(&self) -> &u32 {
        &self.uid
    }

    #[cfg(test)]
    pub(crate) fn from_test_data(uid: u32, data: f32, from_index: usize, to_index: usize) -> Self {
        Self {
            from_index,
            to_index,
            data,
            uid,
        }
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Connection {}

impl Hash for Connection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}
