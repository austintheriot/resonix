use crate::{primitives::Id, traits::GenerateId};

/// A minimal `GenerateId` implementation for unit tests.
/// Yields sequential IDs starting from the initial value.
pub(crate) struct TestIdGenerator(pub usize);

impl GenerateId for TestIdGenerator {
    fn generate_id(&mut self) -> Id {
        let id = self.0;
        self.0 += 1;
        id.into()
    }
}
