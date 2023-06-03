use std::sync::Arc;

use cpal::Sample;

use crate::{AudioPlayerContext, FromContext};

/// Allows any function implementing the following constraints
/// to be called inside the `Player` struct for generating audio--
/// also allows arbitrary arguments, so long as they can be extracted
/// from the audio context
pub trait GetFrame<'c, S, UserData, ExtractedData>
where
    S: Sample,
{
    fn call(&self, buffer: &mut [S], context: Arc<AudioPlayerContext<UserData>>);
}

impl<'c, S, Callback, UserData> GetFrame<'c, S, UserData, ()> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S]),
{
    fn call(&self, buffer: &mut [S], _: Arc<AudioPlayerContext<UserData>>) {
        (self)(buffer);
    }
}

impl<'c, S, Callback, UserData, ExtractedData> GetFrame<'c, S, UserData, (ExtractedData,)>
    for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData),
    ExtractedData: FromContext<UserData>,
{
    fn call(&self, buffer: &mut [S], context: Arc<AudioPlayerContext<UserData>>) {
        (self)(buffer, ExtractedData::from_context(Arc::clone(&context)));
    }
}

impl<'c, S, Callback, UserData, ExtractedData1, ExtractedData2>
    GetFrame<'c, S, UserData, (ExtractedData1, ExtractedData2)> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData1, ExtractedData2),
    ExtractedData1: FromContext<UserData>,
    ExtractedData2: FromContext<UserData>,
{
    fn call(&self, buffer: &mut [S], context: Arc<AudioPlayerContext<UserData>>) {
        (self)(
            buffer,
            ExtractedData1::from_context(Arc::clone(&context)),
            ExtractedData2::from_context(Arc::clone(&context)),
        );
    }
}

impl<'c, S, Callback, UserData, ExtractedData1, ExtractedData2, ExtractedData3>
    GetFrame<'c, S, UserData, (ExtractedData1, ExtractedData2, ExtractedData3)> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData1, ExtractedData2, ExtractedData3),
    ExtractedData1: FromContext<UserData>,
    ExtractedData2: FromContext<UserData>,
    ExtractedData3: FromContext<UserData>,
{
    fn call(&self, buffer: &mut [S], context: Arc<AudioPlayerContext<UserData>>) {
        (self)(
            buffer,
            ExtractedData1::from_context(Arc::clone(&context)),
            ExtractedData2::from_context(Arc::clone(&context)),
            ExtractedData3::from_context(Arc::clone(&context)),
        );
    }
}
