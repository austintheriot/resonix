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
    fn call<'b>(&self, buffer: &'b mut [S], context: &'c AudioPlayerContext<UserData>);
}

impl<'c, S, Callback, UserData> GetFrame<'c, S, UserData, ()> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S]),
{
    fn call<'b>(&self, buffer: &'b mut [S], _: &'c AudioPlayerContext<UserData>) {
        (self)(buffer);
    }
}

impl<'c, S, Callback, UserData, ExtractedData> GetFrame<'c, S, UserData, (ExtractedData,)>
    for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData),
    ExtractedData: FromContext<'c, UserData>,
{
    fn call<'b>(&self, buffer: &'b mut [S], context: &'c AudioPlayerContext<UserData>) {
        (self)(buffer, ExtractedData::from_context(context));
    }
}

impl<'c, S, Callback, UserData, ExtractedData1, ExtractedData2>
    GetFrame<'c, S, UserData, (ExtractedData1, ExtractedData2)> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData1, ExtractedData2),
    ExtractedData1: FromContext<'c, UserData>,
    ExtractedData2: FromContext<'c, UserData>,
{
    fn call<'b>(&self, buffer: &'b mut [S], context: &'c AudioPlayerContext<UserData>) {
        (self)(
            buffer,
            ExtractedData1::from_context(context),
            ExtractedData2::from_context(context),
        );
    }
}

impl<'c, S, Callback, UserData, ExtractedData1, ExtractedData2, ExtractedData3>
    GetFrame<'c, S, UserData, (ExtractedData1, ExtractedData2, ExtractedData3)> for Callback
where
    S: Sample,
    Callback: Fn(&mut [S], ExtractedData1, ExtractedData2, ExtractedData3),
    ExtractedData1: FromContext<'c, UserData>,
    ExtractedData2: FromContext<'c, UserData>,
    ExtractedData3: FromContext<'c, UserData>,
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
