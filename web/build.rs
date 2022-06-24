use common::utils;
use rodio::Decoder;
use std::env;
use std::fs::{self, File, ReadDir};
use std::path::Path;

pub const DEFAULT_AUDIO_FILE_INDEX: usize = 3;

fn get_audio_read_dir() -> ReadDir {
    fs::read_dir("../audio").unwrap()
}

// generates a list of audio files from the `audio` directory at compile time,
// which is then available in the app, so that default buffer lists are always
// in sync with the actual `audio` directory
fn build_audio_files_list() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("audio_files.rs");
    let paths = get_audio_read_dir();
    let num_paths = paths.count();

    let paths = get_audio_read_dir();
    // get a basic list of file names from the `audio` directory
    let paths: Vec<_> = paths.map(|file| file.unwrap().file_name()).collect();
    // convert file names into strings with quotes
    let paths: Vec<_> = paths.iter().map(|path| format!(r#"{:?}"#, path)).collect();
    // format file names with commas between them
    let paths = paths.join(", ");

    let audio_file_string = format!(
        r#"
        pub const AUDIO_FILES:  [&'static str; {}] = [{}];

        pub const DEFAULT_AUDIO_FILE_INDEX: usize = {};

        pub const DEFAULT_AUDIO_FILE: &str = AUDIO_FILES[DEFAULT_AUDIO_FILE_INDEX];
        "#,
        num_paths, paths, DEFAULT_AUDIO_FILE_INDEX
    );

    fs::write(&dest_path, audio_file_string).unwrap();
}

/// Pre-decodes initial audio data buffer so that the initial load can be as quick as possible
fn build_decoded_audio_buffer() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("audio_decoded.rs");
    let input_audio_path = get_audio_read_dir()
        .into_iter()
        .enumerate()
        .find(|(i, _)| *i == DEFAULT_AUDIO_FILE_INDEX)
        .unwrap()
        .1
        .unwrap()
        .path();
    let input_audio_file = File::open(input_audio_path).unwrap();
    let mp3_source = Decoder::new(input_audio_file).unwrap();
    let mp3_source_data: Vec<f32> = utils::i16_array_to_f32(mp3_source.collect());
    let encoded: Vec<u8> = bincode::serialize(&mp3_source_data).unwrap();

    fs::write(&dest_path, encoded).unwrap();
}

fn main() {
    build_audio_files_list();
    build_decoded_audio_buffer();

    // println!("cargo:rerun-if-changed=../audio");
}
