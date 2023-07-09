use std::hash::{Hash, Hasher};

use resonix_core::NumChannels;

pub type ConnectionUid = u32;

#[derive(Debug, Clone)]
pub struct Connection {
    /// where the connection is coming from
    from_index: usize,
    /// where the connection is going to
    to_index: usize,
    /// the data that the connection is carrying (if any)
    data: Vec<f32>,
    num_channels: NumChannels,
    uid: ConnectionUid,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            data: vec![0.0],
            from_index: 0,
            to_index: 0,
            uid: 0,
            num_channels: NumChannels::from(0),
        }
    }
}

impl Connection {
    pub fn new(num_channels: impl Into<NumChannels>) -> Self {
        Self::from_indexes(num_channels, 0, 0)
    }

    pub fn from_indexes(
        num_channels: impl Into<NumChannels>,
        from_index: usize,
        to_index: usize,
    ) -> Self {
        let num_channels = num_channels.into();
        Self {
            num_channels,
            data: vec![0.0; *num_channels],
            from_index,
            to_index,
            uid: 0,
        }
    }

    pub fn num_channels(&self) -> NumChannels {
        self.num_channels
    }

    pub fn from_index(&self) -> usize {
        self.from_index
    }

    pub fn to_index(&self) -> usize {
        self.to_index
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Replaces inner data with a new vector
    ///
    /// Warning: this is likely expensive. Prefer `update_data` to modify values in-place
    pub fn set_data(&mut self, data: Vec<f32>) -> &mut Self {
        self.data = data;
        self
    }

    /// Updates inner data in-place
    pub fn update_data(&mut self, f: impl Fn(&mut [f32])) -> &mut Self {
        (f)(&mut self.data);
        self
    }

    pub fn uid(&self) -> &ConnectionUid {
        &self.uid
    }

    #[cfg(test)]
    pub(crate) fn from_test_data(
        uid: ConnectionUid,
        num_channels: impl Into<NumChannels>,
        data: Vec<f32>,
        from_index: usize,
        to_index: usize,
    ) -> Self {
        let num_channels = num_channels.into();
        Self {
            num_channels,
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
