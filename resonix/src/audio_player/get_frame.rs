use cpal::Sample;

use crate::{AudioPlayerContext, FromContext};

/// Allows any function implementing the following constraints
/// to be called inside the `Player` struct for generating audio--
/// also allows arbitrary arguments, so long as they can be extracted
/// from the audio context
pub trait GetFrame<S: Sample, T> {
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext);
}

impl<S, F> GetFrame<S, ()> for F
where
    S: Sample,
    F: Fn(&mut [S]),
{
    fn call(self, buffer: &mut [S], _: &AudioPlayerContext) {
        (self)(buffer);
    }
}

impl<S, F, T> GetFrame<S, T> for F
where
    S: Sample,
    F: Fn(&mut [S], T),
    T: FromContext,
{
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext) {
        (self)(buffer, T::from_context(context));
    }
}

impl<S, F, T1, T2> GetFrame<S, (T1, T2)> for F
where
    S: Sample,
    F: Fn(&mut [S], T1, T2),
    T1: FromContext,
    T2: FromContext,
{
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext) {
        (self)(buffer, T1::from_context(context), T2::from_context(context));
    }
}

impl<S, F, T1, T2, T3> GetFrame<S, (T1, T2, T3)> for F
where
    S: Sample,
    F: Fn(&mut [S], T1, T2, T3),
    T1: FromContext,
    T2: FromContext,
    T3: FromContext,
{
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext) {
        (self)(
            buffer,
            T1::from_context(context),
            T2::from_context(context),
            T3::from_context(context),
        );
    }
}

impl<S, F, T1, T2, T3, T4> GetFrame<S, (T1, T2, T3, T4)> for F
where
    S: Sample,
    F: Fn(&mut [S], T1, T2, T3, T4),
    T1: FromContext,
    T2: FromContext,
    T3: FromContext,
    T4: FromContext,
{
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext) {
        (self)(
            buffer,
            T1::from_context(context),
            T2::from_context(context),
            T3::from_context(context),
            T4::from_context(context),
        );
    }
}

impl<S, F, T1, T2, T3, T4, T5> GetFrame<S, (T1, T2, T3, T4, T5)> for F
where
    S: Sample,
    F: Fn(&mut [S], T1, T2, T3, T4, T5),
    T1: FromContext,
    T2: FromContext,
    T3: FromContext,
    T4: FromContext,
    T5: FromContext,
{
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext) {
        (self)(
            buffer,
            T1::from_context(context),
            T2::from_context(context),
            T3::from_context(context),
            T4::from_context(context),
            T5::from_context(context),
        );
    }
}

impl<S, F, T1, T2, T3, T4, T5, T6> GetFrame<S, (T1, T2, T3, T4, T5, T6)> for F
where
    S: Sample,
    F: Fn(&mut [S], T1, T2, T3, T4, T5, T6),
    T1: FromContext,
    T2: FromContext,
    T3: FromContext,
    T4: FromContext,
    T5: FromContext,
    T6: FromContext,
{
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext) {
        (self)(
            buffer,
            T1::from_context(context),
            T2::from_context(context),
            T3::from_context(context),
            T4::from_context(context),
            T5::from_context(context),
            T6::from_context(context),
        );
    }
}

impl<S, F, T1, T2, T3, T4, T5, T6, T7> GetFrame<S, (T1, T2, T3, T4, T5, T6, T7)> for F
where
    S: Sample,
    F: Fn(&mut [S], T1, T2, T3, T4, T5, T6, T7),
    T1: FromContext,
    T2: FromContext,
    T3: FromContext,
    T4: FromContext,
    T5: FromContext,
    T6: FromContext,
    T7: FromContext,
{
    fn call(self, buffer: &mut [S], context: &AudioPlayerContext) {
        (self)(
            buffer,
            T1::from_context(context),
            T2::from_context(context),
            T3::from_context(context),
            T4::from_context(context),
            T5::from_context(context),
            T6::from_context(context),
            T7::from_context(context),
        );
    }
}
