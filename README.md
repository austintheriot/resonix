# Rust Audio

## About 

This repo is a very WIP investigation into using Rust for creative coding on various platforms (e.g. desktop, web, etc.). Currently, the only demonstration that is even remotely in a working state is the web one, which still needs quite a bit of work to improve its performance.

## Todos:
- add grain length sliders
- Add an audio loading / initialization state style / animation

- Move audio PROCESSING into a Web Worker thread so that `cpal` merely has to request audio data at the appropriate time
    - Keeps main thread / audio processing from getting locked up by UI updates and vice versa
    - Keeps WebAudio context on the main thread (where it has to be)

- Share a single audio context that is initialized (?) at init time?
- Memoize decoded audio from previous files? To prevent stutter on change?
- Enable draggging the current buffer selection window?

- Web
    - Refactor visual representation of current audio buffer:
        - use an svg <path /> element?
        - probably would be best to use a canvas to do this
    - Clean up logic around buffer selection ranges -- ensure no empty ranges?

- Common
    - Clean up any unused files in /common
    - Move audio handles in there?

- CLI
    - Re-sample any audio files that don't match the current sample rate.

- Native
    - Make a Native app using Tauri that relies on current web view with Rust running natively under the hood?
    - Use `serde_wasm_bindgen` instead of message passing between backend and front end to prevent JSON-ifying buffers?

- More audio tools / effects:
    - Recording
    - Reverb
    - Delay

Visual effects:
    - WebGL: particles that react / correspond to audio grains
    - Show audio output as a sample window?
    - Or just show current amplitude output with simple bars

--------------------------

General Fixes:
 - Correct sample rate for mp3 audio for all environments.