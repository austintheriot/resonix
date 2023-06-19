#[derive(Copy, Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct Connection {
    /// where the connection is coming from
    pub from_index: usize,
    /// where the connection is going to
    pub to_index: usize,
    /// the data that the connection is carrying (if any)
    pub data: f32,
    /// used while building a dependency graph of audio connections
    /// if `true`, then that connection has been provided data by
    /// parent node
    pub init: bool,
}

impl Connection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_indexes(from_index: usize, to_index: usize) -> Self {
        Self { from_index, to_index, data: 0.0, init: false, }
    }
}