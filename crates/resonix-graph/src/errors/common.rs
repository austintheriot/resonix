use thiserror::Error;

#[derive(Debug, Error)]
#[error("Internal error: buffer already allocated")]
pub struct BufferAlreadyAllocated;

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn display_message_matches_expected_text() {
        let error = BufferAlreadyAllocated;
        assert_eq!(
            error.to_string(),
            "Internal error: buffer already allocated"
        );
    }
}
