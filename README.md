# Resonix

## About

This repo is my investigation into using Rust for creative, cross-platform audio programming.

[Granular Synthesizer Demo](https://austintheriot.github.io/resonix/)

![Granular Synthesizer Demo](/screenshots/granular_synthesizer_0.png)

## Dependencies

On Linux, `alsa` or `jack` required before building/running any examples that require native audio device support (as opposed to WASM).

Install `alsa` like so:

```sh
sudo apt install libasound2-dev
```

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in resonix by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
