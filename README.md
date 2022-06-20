# Rust Audio

## Todos:
- Web
    - Make play/pause/load separate actions & events:
        - Initialize audio right away
        - Make play / pause use the current_status as the default mechanism for starting/stopping playback
        - Do not include mp3 data in binary: fetch at runtime once a sound file has been selected (fetch default at runtime too)

    - Build Interactive UI
        - Refactor visual representation of current audio buffer to use an svg <path /> element?
        - Create a dropdown for switching out audio buffers
        - Create sliders/knobs for adjust grain length
        - Make nicer styles
    - Fixes:
        - Clean up logic around buffer selection ranges -- ensure no empty ranges


- Refactor for shared code in `/common`
    - Generalize multi-channel mixing for any number of runtime channels
    - Extract logic for mixing down multi-channel audio into lower-channel audio

- CLI
    - Re-sample any audio files that don't match the current sample rate.

- Add reverb (and other effects)


--------------------------

General Fixes:

 - Correct sample rate for mp3 audio for both CLI and web.