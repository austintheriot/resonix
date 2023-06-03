use std::sync::Arc;

use cpal::Sample;

use crate::{AudioPlayerContext, FromContext};

/// Allows any function implementing the following constraints
/// to be called inside the `Player` struct for generating audio--
/// also allows arbitrary arguments, so long as they can be extracted
/// from the audio context
pub trait GetFrame<'a, 'c: 'a, S, UserData, ExtractedData>
where
    S: Sample,
{
    fn call<'b>(&self, buffer: &'b mut [S], context: &'c AudioPlayerContext<UserData>);
}

impl<'a, 'c: 'a, S, Callback, UserData> GetFrame<'a, 'c, S, UserData, ()> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S]),
{
    fn call<'b>(&self, buffer: &'b mut [S], _: &'c AudioPlayerContext<UserData>) {
        (self)(buffer);
    }
}

impl<'a, 'c: 'a, S, Callback, UserData: 'a, ExtractedData>
    GetFrame<'a, 'c, S, UserData, (ExtractedData,)> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData),
    ExtractedData: FromContext<'a, UserData>,
{
    fn call<'b>(&self, buffer: &'b mut [S], context: &'c AudioPlayerContext<UserData>) {
        (self)(buffer, ExtractedData::from_context(context));
    }
}

impl<'a, 'c: 'a, S, Callback, UserData: 'a, ExtractedData1, ExtractedData2>
    GetFrame<'a, 'c, S, UserData, (ExtractedData1, ExtractedData2)> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData1, ExtractedData2),
    ExtractedData1: FromContext<'a, UserData>,
    ExtractedData2: FromContext<'a, UserData>,
{
    fn call<'b>(&self, buffer: &'b mut [S], context: &'c AudioPlayerContext<UserData>) {
        (self)(
            buffer,
            ExtractedData1::from_context(context),
            ExtractedData2::from_context(context),
        );
    }
}

impl<'a, 'c: 'a, S, Callback, UserData: 'a, ExtractedData1, ExtractedData2, ExtractedData3>
    GetFrame<'a, 'c, S, UserData, (ExtractedData1, ExtractedData2, ExtractedData3)> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData1, ExtractedData2, ExtractedData3),
    ExtractedData1: FromContext<'a, UserData>,
    ExtractedData2: FromContext<'a, UserData>,
    ExtractedData3: FromContext<'a, UserData>,
{
    fn call<'b>(&self, buffer: &'b mut [S], context: &'c AudioPlayerContext<UserData>) {
        (self)(
            buffer,
            ExtractedData1::from_context(context),
            ExtractedData2::from_context(context),
            ExtractedData3::from_context(context),
        );
    }
}
