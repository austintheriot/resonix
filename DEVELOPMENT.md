# Development

# Production build

For building all wasm targets in production mode, all js libraries that depend on that wasm output, and also all examples that consume the build packages:

```sh
yarn build
```

# Watch Mode

Rebuild Rust & Wasm library code on Rust code changes:

```sh
yarn dev:rust
```

Rebuild JS library wrapper code around Rust/Wasm on changes:

```
cd crates/resonix-web-js
yarn dev:js
```

Run local dev server for web vite example app

```
cd crates/resonix-web-js
yarn dev:js
```
