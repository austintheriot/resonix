# Rust Audio

## Todos:
- Fixes:
    - Correct sample rate for mp3 audio for both CLI and web.

- Refactor for shared code in `/common`
    - Generalize multi-channel mixing for any number of runtime channels
    - Extract logic for mixing down multi-channel audio into lower-channel audio

- CLI
    - Re-sample any audio files that don't match the current sample rate.

- Web
    - Build Interactive UI
        - Show visual representation of current audio buffer
        - Create a dropdown for switching out audio buffers
        - Allow slidable / interactive window for sampling grains

- Add reverb (and other effects)