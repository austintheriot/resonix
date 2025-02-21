/// A uid that is unique to the current audio context
pub trait AudioContextUid {
    fn from_usize(u: usize) -> Self;
}
