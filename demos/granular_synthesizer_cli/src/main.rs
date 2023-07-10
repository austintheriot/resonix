use resonix::{
    granular_synthesizer::GranularSynthesizer, granular_synthesizer::GranularSynthesizerAction,
    AudioContext, DACNode, DownmixNode, Downmixer, GranularSynthesizerNode,
};
use rodio::{Decoder, Source};
use std::{sync::Arc, time::Duration};

/// Converts default mp3 file to raw audio sample data
fn load_default_buffer() -> Arc<Vec<f32>> {
    // get audio file data as compile time
    let audio_file_slice =
        std::io::Cursor::new(include_bytes!("../../../assets/ecce_nova_3.mp3").as_ref());
    let mp3_source = Decoder::new(audio_file_slice).unwrap();
    let num_channels = mp3_source.channels() as usize;
    let mp3_source_data: Vec<f32> =
        granular_synthesizer_cli::utils::i16_array_to_f32(mp3_source.collect());
    let left_channel_audio_data = mp3_source_data.into_iter().step_by(num_channels).collect();

    Arc::new(left_channel_audio_data)
}

#[tokio::main]
pub async fn main() {
    let mut audio_context = AudioContext::new();

    let mut granular_synthesizer = GranularSynthesizer::new();
    granular_synthesizer
        .set_buffer(load_default_buffer())
        .set_grain_len(Duration::from_millis(1000))
        .set_num_channels(50);
    let granular_synthesizer_node = GranularSynthesizerNode::new(granular_synthesizer);
    let downmix_node = DownmixNode::new(50, 2, Downmixer::Panning);
    let dac_node = DACNode::new(2);

    let granular_synthesizer_handle = audio_context.add_node(granular_synthesizer_node).unwrap();
    let downmix_node_handle = audio_context.add_node(downmix_node).unwrap();
    let dac_node_handle = audio_context.add_node(dac_node).unwrap();

    audio_context
        .connect(granular_synthesizer_handle, downmix_node_handle)
        .unwrap();
    audio_context
        .connect(downmix_node_handle, dac_node_handle)
        .unwrap();

    let _audio_context = audio_context.into_audio_init().unwrap();

    tokio::time::sleep(Duration::MAX).await;
}
