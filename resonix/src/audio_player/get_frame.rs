use std::sync::Arc;

use cpal::Sample;

use crate::{AudioPlayerContext, UserDataFromContext};

/// Allows any function implementing the following constraints
/// to be called inside the `Player` struct for generating audio--
/// also allows arbitrary arguments, so long as they can be extracted
/// from the audio context
pub trait GetFrame<S, UserData, ExtractedData>
where
    S: Sample,
{
    fn call(&self, buffer: &mut [S], context: Arc<AudioPlayerContext<UserData>>);
}

// macro example:
//
// impl<S, Callback, UserData, ExtractedData> GetFrame<S, UserData, (ExtractedData,)> for Callback
// where
//     S: Sample,
//     Callback: Fn(&mut [S], ExtractedData),
//     ExtractedData: UserDataFromContext<UserData>,
// {
//     fn call(&self, buffer: &mut [S], context: Arc<AudioPlayerContext<UserData>>) {
//         (self)(buffer, ExtractedData::from_context(Arc::clone(&context)));
//     }
// }

macro_rules! impl_get_frame {
    (
        $($param:ident),*
    ) => {
        #[allow(non_snake_case)]
        #[allow(unused)]
        impl<
        S, Callback, UserData, $($param, )*
        >
            GetFrame<
            S, UserData, ($($param, )*)
            >
            for Callback
            where
                S: Sample,
                Callback: Fn(&mut [S], $($param, )*),
                $($param: UserDataFromContext<UserData>,)*
        {
            fn call(&self, buffer: &mut [S], context: Arc<AudioPlayerContext<UserData>>) {
                (self)(buffer, $(
                    $param::from_context(Arc::clone(&context)),
                )*)
                ;
            }
        }
    }
}

impl_get_frame!();
impl_get_frame!(E1);
impl_get_frame!(E1, E2);
impl_get_frame!(E1, E2, E3);
impl_get_frame!(E1, E2, E3, E4);
impl_get_frame!(E1, E2, E3, E4, E5);
impl_get_frame!(E1, E2, E3, E4, E5, E6);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14, E15);
impl_get_frame!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14, E15, E16);
