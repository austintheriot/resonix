use crate::SystemAudioOutputError;

#[cfg(feature = "mock")]
use crate::traits::Consumer;
#[cfg(feature = "mock")]
use alloc::boxed::Box;

pub trait SystemAudioOutput<S: Copy> {
    fn try_write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError>;

    fn try_write_block(&mut self, samples: &[S]) -> Result<(), SystemAudioOutputError> {
        for sample in samples {
            self.try_write_sample(*sample)?;
        }

        Ok(())
    }

    fn ready_for_sample(&self) -> bool;

    #[cfg(feature = "mock")]
    fn consumer(&mut self) -> Option<Box<dyn Consumer<S> + Send>>;
}
