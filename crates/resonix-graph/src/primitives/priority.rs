use core::ops::Deref;

/// Strongly typed wrapper around a usize for better type checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(usize);

impl Priority {
    pub const fn new(priority: usize) -> Self {
        Self(priority)
    }
}

// other convenience implementations possible here

impl Deref for Priority {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<usize>> From<I> for Priority {
    fn from(value: I) -> Self {
        Priority(value.into())
    }
}
