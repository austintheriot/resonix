use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub(crate) struct ConnectionInner {
    /// where the connection is coming from
    pub(crate) from_index: usize,
    /// where the connection is going to
    pub(crate) to_index: usize,
    /// the data that the connection is carrying (if any)
    pub(crate) data: f32,
    /// used while building a dependency graph of audio connections
    /// if `true`, then that connection has been provided data by
    /// parent node
    pub(crate) init: bool,
}

impl ConnectionInner {
    pub(crate) fn new() -> Self {
        Self::from_indexes(0, 0)
    }

    pub(crate) fn from_indexes(from_index: usize, to_index: usize) -> Self {
        Self {
            from_index,
            to_index,
            data: 0.0,
            init: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Connection {
    connection_inner: Arc<Mutex<ConnectionInner>>,
    pub uuid: Uuid,
}

impl Connection {
    pub fn new() -> Self {
        Self::from_indexes(0, 0)
    }

    pub fn from_indexes(from_index: usize, to_index: usize) -> Self {
        Self {
            connection_inner: Arc::new(Mutex::new(ConnectionInner::from_indexes(
                from_index, to_index,
            ))),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn from_index(&self) -> usize {
        self.connection_inner.lock().unwrap().from_index
    }

    pub fn to_index(&self) -> usize {
        self.connection_inner.lock().unwrap().to_index
    }

    pub fn data(&self) -> f32 {
        self.connection_inner.lock().unwrap().data
    }

    pub fn init(&self) -> bool {
        self.connection_inner.lock().unwrap().init
    }

    pub fn set_data(&mut self, data: f32) -> &mut Self {
        self.connection_inner.lock().unwrap().data = data;
        self
    }

    pub fn set_init(&mut self, init: bool) -> &mut Self {
        self.connection_inner.lock().unwrap().init = init;
        self
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// for easier testing
    #[cfg(test)]
    pub(crate) fn from_connection_inner(connection_inner: ConnectionInner) -> Self {
        Self {
            connection_inner: Arc::new(Mutex::new(connection_inner)),
            uuid: Uuid::new_v4(),
        }
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
