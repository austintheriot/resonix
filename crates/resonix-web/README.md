# Running Tests

With wasm-pack

```sh
wasm-pack test --headless --chrome
```

Without wasm-pack

```sh
cargo test --target wasm32-unknown-unknown
```

See https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html
