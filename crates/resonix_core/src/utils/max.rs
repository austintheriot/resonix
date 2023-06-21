/// Custom trait for comparison of [Percentage](crate::Percentage) structs
pub trait Max {
    type Rhs;
    type Output;
    fn max(self, rhs: Self::Rhs) -> Self::Output;
}
