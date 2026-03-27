# Building Wasm Targets

For building all wasm targets from workspace root:

```sh
# only including example for resonix_graph right now, but other
# crates may be (more) relevant soon
cargo run -p build-wasm -- --wasm-name resonix
```
