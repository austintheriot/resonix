use rodio::Sink;
use rodio::source::SineWave;
use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

fn main() {
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open("src/pater_emon.mp3").unwrap());
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap();
    // Play the sound directly on the device
    stream_handle
        .play_raw(source.convert_samples())
        .expect("Should play raw audio");

    let sink = Sink::try_new(&stream_handle).unwrap();

    // Add a dummy source of the sake of the example.
    let source = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(2.0))
        .amplify(0.20);

    sink.append(source);

    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    sink.sleep_until_end();
}
