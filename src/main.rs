use rodio::{Sink};
use std::f32::consts::PI;

const SAMPLE_RATE: usize = 44100;
const SECONDS_OF_PLAYBACK: usize = 5;

fn get_sample_at_frequency(frame: usize, frequency: f32,) -> f32 {
    f32::sin((frame as f32 * frequency * PI * 2.) / SAMPLE_RATE as f32)
}

fn main() {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let mut buffer_vec = Vec::from([0.; SAMPLE_RATE * SECONDS_OF_PLAYBACK]);

    for (frame_index, frame) in buffer_vec.iter_mut().enumerate() {
        let ampplitude_1 = get_sample_at_frequency(frame_index, 440.);
        let amplitude_2 = get_sample_at_frequency(frame_index, 157.);
        *frame = ampplitude_1 * amplitude_2;
    }
    
    use rodio::buffer::SamplesBuffer;
    let buffer = SamplesBuffer::new(1, 44100, buffer_vec);
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(buffer);
    sink.sleep_until_end();
}
