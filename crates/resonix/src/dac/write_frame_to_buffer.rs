use std::sync::Arc;

use cpal::Sample;

use crate::{DACConfig, DataFromDACConfig};

/// Allows any function implementing the following constraints
/// to be called inside the `Player` struct for generating audio--
/// also allows arbitrary arguments, so long as they can be extracted
/// from the audio config
pub trait WriteFrameToBuffer<S, ExtractedData>
where
    S: Sample,
{
    fn call(&mut self, buffer: &mut [S], config: Arc<DACConfig>);
}

macro_rules! impl_write_frame_to_bufer {
    (
        $($param:ident),*
    ) => {
        #[allow(non_snake_case)]
        #[allow(unused)]
        impl<
        S, Callback, $($param, )*
        >
            WriteFrameToBuffer<
            S, ($($param, )*)
            >
            for Callback
            where
                S: Sample,
                Callback: FnMut(&mut [S], $($param, )*),
                $($param: DataFromDACConfig,)*
        {
            fn call(&mut self, buffer: &mut [S], config: Arc<DACConfig>) {
                (self)(buffer, $(
                    $param::from_config(Arc::clone(&config)),
                )*)
                ;
            }
        }
    }
}

impl_write_frame_to_bufer!();
impl_write_frame_to_bufer!(E1);
impl_write_frame_to_bufer!(E1, E2);
impl_write_frame_to_bufer!(E1, E2, E3);
impl_write_frame_to_bufer!(E1, E2, E3, E4);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14, E15);
impl_write_frame_to_bufer!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14, E15, E16);
