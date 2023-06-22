use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct Connection {
    /// where the connection is coming from
    pub(crate) from_index: usize,
    /// where the connection is going to
    pub(crate) to_index: usize,
    /// the data that the connection is carrying (if any)
    pub(crate) data: f32,
    pub uuid: Uuid,
}

impl Connection {
    pub fn new() -> Self {
        Self::from_indexes(0, 0)
    }

    pub fn from_indexes(from_index: usize, to_index: usize) -> Self {
        Self {
            data: 0.0,
            from_index: 0,
            to_index: 0,
            uuid: Uuid::new_v4(),
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

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Connection {}

impl Hash for Connection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}
