# Resonix

[![CI Image]][wrend CI]

[CI Image]: https://img.shields.io/github/actions/workflow/status/austintheriot/wrend/ci.yml?branch=master

[wrend CI]: https://github.com/austintheriot/wrend/actions/workflows/ci.yml

**This library is currently in early development**. Feel free to use it, but do so with the knowledge that APIs are likely to change without consideration for backwards compatibility.

## About

This repo is my investigation into using Rust for creative, cross-platform audio programming.

[Granular Synthesizer Demo](https://austintheriot.github.io/resonix/)

![Granular Synthesizer Demo](/screenshots/granular_synthesizer_0.png)

## Dependencies

On Linux, `alsa` or `jack` is required before building/running any examples that require native audio device support (as opposed to WASM).

Install `alsa` like so on Ubuntu:

```sh
sudo apt-get install libasound2-dev
```

or

```sh
sudo apt-get install libjack-jackd2-dev libjack-jackd2-0
```

## License

Licensed under either of [Apache License, Version
2.0](LICENSE-APACHE) or [MIT license](LICENSE_mit) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in resonix by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.