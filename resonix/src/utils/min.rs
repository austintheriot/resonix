/// Custom trait for comparison of [Percentage](crate::Percentage) structs
pub trait Min {
    type Rhs;
    type Output;
    fn min(self, rhs: Self::Rhs) -> Self::Output;
}
