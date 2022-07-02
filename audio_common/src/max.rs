pub trait Max {
    type Rhs;
    type Output;
    fn max(self: Self, rhs: Self::Rhs) -> Self::Output;
}
