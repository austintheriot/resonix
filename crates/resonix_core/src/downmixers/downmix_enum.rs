use crate::{
    downmix_panning, downmix_panning_fast, downmix_panning_fast_to_buffer,
    downmix_panning_to_buffer, downmix_simple, downmix_simple_to_buffer,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Downmixer {
    PanningFast,
    Panning,
    #[default]
    Simple,
}

impl Downmixer {
    pub fn as_downmixer(&self) -> impl Fn(&[f32], u32) -> Vec<f32> {
        match self {
            Downmixer::PanningFast => downmix_panning_fast,
            Downmixer::Panning => downmix_panning,
            Downmixer::Simple => downmix_simple,
        }
    }

    pub fn as_downmixer_to_buffer(
        &self,
    ) -> impl for<'a> Fn(&[f32], u32, &'a mut [f32]) -> &'a mut [f32] {
        match self {
            Downmixer::PanningFast => downmix_panning_fast_to_buffer,
            Downmixer::Panning => downmix_panning_to_buffer,
            Downmixer::Simple => downmix_simple_to_buffer,
        }
    }
}
