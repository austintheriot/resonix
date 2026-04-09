# Development

For building all wasm targets & js examples:

```sh
yarn build
```

# Watch Mode

Rebuild Rust & Wasm library code on Rust code changes:

```sh
# ignore JS-only crates, only build `bundler` target (the only one we consume in the js library for now)
cargo watch -w ./crates -i ./crates/resonix-web-js \
    -- cargo run -p build-wasm \
    -- --target bundler
```

Rebuild JS library wrapper code around Rust/Wasm on changes:

```
cd crates/resonix-web-js
yarn dev
```
