# Setup

- Install rust using [rustup](https://www.rust-lang.org/tools/install)

- If Rust is already installed, update to the latest version

```sh
rustup update
```

- Install the wasm32-unknown-unknown target (for compiling Rust code to .wasm)

```sh
rustup target add wasm32-unknown-unknown
```

- Install wasm-bindgen-cli (for generating the JS bindings to use the .wasm files in the browser)

Note: must match version of `wasm-bindgen` installed [locally](./Cargo.toml).

```sh
cargo install wasm-bindgen-cli --vers "0.2.100"
```

- Install Firefox's `geckodriver` to your $PATH (for running wasm tests in headless mode): https://github.com/mozilla/geckodriver/releases

`geckodriver` must be in your $PATH, so that it can be executed from `cargo make`.

- Install cargo-make (for running watch/test scripts)

```sh
cargo install cargo-make
```

- Install cargo-watch (for rebuilding on edits to Rust source files)

```sh
cargo install cargo-watch
```
