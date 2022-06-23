use std::env;
use std::fs::{self, ReadDir};
use std::path::Path;

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
        r#"pub const AUDIO_FILES:  [&'static str; {}] = [{}];"#,
        num_paths, paths
    );

    fs::write(&dest_path, audio_file_string).unwrap();
}

fn main() {
    build_audio_files_list();

    println!("cargo:rerun-if-changed=../audio");
}
