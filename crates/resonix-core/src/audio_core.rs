use wasm_bindgen::prelude::wasm_bindgen;

use crate::{AudioFrameOutputs, audio_frame_inputs::AudioFrameInputs};

#[wasm_bindgen]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AudioCore;

impl AudioCore {
    pub fn compute_audio_frame(_audio_frame_inputs: AudioFrameInputs) -> AudioFrameOutputs {
        AudioFrameOutputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_computes_an_audio_frame() {
        let audio_frame_inputs = AudioFrameInputs;
        let audio_frame_outputs = AudioCore::compute_audio_frame(audio_frame_inputs);
        assert_eq!(audio_frame_outputs, AudioFrameOutputs);
    }
}
