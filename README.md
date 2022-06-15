# Rust Audio

## Todos:
- CLI
    - Re-sample any audio files that don't match the current sample rate.

- Web
    - Build Interactive UI

- Refactor for shared code in `/common`
    - Extract Granular Synth logic: produce an audio stream that can be consuemd in `next_value`
    - Extract logic for mixing down multi-channel audio into lower-channel audio

- Add reverb (and other effects)