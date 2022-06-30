pub trait Min {
    type Rhs;
    type Output;
    fn min(self: Self, rhs: Self::Rhs) -> Self::Output;
}