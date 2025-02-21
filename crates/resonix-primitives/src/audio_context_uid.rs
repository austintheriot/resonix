pub struct AudioContextUid(pub(crate) usize);

impl AudioContextUid {
    pub(crate) fn new(uid: usize) -> Self {
        Self(uid)
    }
}

impl resonix_types::AudioContextUid for AudioContextUid {
    fn from_usize(u: usize) -> Self {
        Self(u)
    }
}
