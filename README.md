# Rust Audio

## Todos:
- Style the HtmlSelectElement for buffer selection
- Move audio PROCESSING into a Web Worker thread so that `cpal` merely has to request audio data at the appropriate time
    - Keeps main thread / audio processing from getting locked up by UI updates and vice versa
    - Keeps WebAudio context on the main thread (where it has to be)
- add grain length sliders
- Make buffer selector cursor:pointer
- Add an audio loading / initialization state style / animation
- Make initial load much faster - save a raw audio Vec for direct access at initialization

- Share a single audio context that is initialized (?) at init time?
- Memoize decoded audio from previous files? To prevent stutter on change?
- Enable draggging the current buffer selection window?
- Fix HtmlSelectElement UI interaction:
    - Require a form submit
    - Make default audio the currently selected file in the select element
    - Disable when audio has not yet been enabled

- Web
    - Refactor visual representation of current audio buffer to use an svg <path /> element?
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