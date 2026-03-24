use alloc::vec::Vec;

use crate::ConsumerError;

pub trait Consumer<S> {
    /// Attempts to read from the Producer
    fn try_read(&mut self) -> Result<S, ConsumerError>;

    /// Reads as many samples as possible into `buf`,
    /// until an error is reached.
    ///
    /// Returns the number of samples written.
    fn read_into(&mut self, buffer: &mut [S]) -> usize {
        let mut count = 0;

        for slot in buffer.iter_mut() {
            match self.try_read() {
                Ok(sample) => {
                    *slot = sample;
                    count += 1;
                }
                Err(_) => break,
            }
        }

        count
    }

    /// Drains all currently available samples into a Vec,
    /// until an error is reached.
    fn drain(&mut self) -> Vec<S> {
        let mut out = Vec::new();

        while let Ok(sample) = self.try_read() {
            out.push(sample);
        }

        out
    }
}
