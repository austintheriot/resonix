# Rust Audio

## About 

This repo is my investigation into using Rust for creative audio coding on various platforms (e.g. desktop, web, etc.), but especially for targeting the web.

[Granular Synthesizer Demo](https://austintheriot.github.io/audio/)

![Granular Synthesizer Demo](/screenshots/granular_synthesizer_0.png)

## Todos:
- Update licenses / cargo tomls, directory names

- Show hoverable tooltips for icon buttons

- Enable sampling from multi-channel audio input

- split granular synth into its own repo / separate website entry before adding more features?

- Show recording buffer visualization (or at least some sort of indication recording is taking place)

- Interpolate changes in Gain

- Name downloaded file something more like: "name_of_audio_granulated.wav" or similar

- Optimizations:
    - Add `finished` grains to a hashmap when finished to prevent the need for searching for spent grains on every frame
    - Improve which `rand` function we're using for better efficiency

- Use newtype-style units to disambiguate calculations

- Move audio PROCESSING into a Web Worker thread so that `cpal` merely has to request audio data at the appropriate time
    - Keeps main thread / audio processing from getting locked up by UI updates and vice versa
    - Keeps WebAudio context on the main thread (where it has to be)

- Share a single audio context that is initialized (?) at init time?
- Memoize decoded audio from previous files? To prevent stutter on change?
- Enable dragging the current buffer selection window?

- Web
    - Refactor visual representation of current audio buffer:
        - use an svg <path /> element?
        - probably would be best to use a canvas to do this

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