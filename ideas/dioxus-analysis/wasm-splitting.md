# WASM Bundle Splitting in Dioxus

Investigation of `dioxus` as of April 2026.

## Overview

Dioxus implements WASM code splitting that breaks a monolithic WASM binary into lazily-loaded chunks. Functions annotated with `#[wasm_split(module_name)]` are compiled into separate `.wasm` files that are fetched on demand. The system has four layers:

1. **Compile-time macros** — mark split boundaries, generate FFI symbols
2. **Binary splitting CLI** — analyzes call graph, produces split modules
3. **JavaScript loader** — fetches and instantiates modules at runtime
4. **Rust async runtime** — suspends and resumes futures around module loads

---

## Layer 1: Compile-Time Macros

**File:** `packages/wasm-split/wasm-split-macro/src/lib.rs`

### `#[wasm_split(module_name)]`

This attribute macro transforms an async function into a lazy-loadable entry point:

```rust
#[wasm_split(my_feature)]
async fn load_feature() -> i32 {
    expensive_computation()
}
```

The macro:

1. Validates the function is `async` (panics otherwise)
2. Generates a stable hash from the function name + token span using SHA-256 → first 16 bytes → base16 → 32-char hex
3. Generates three components using a naming scheme the CLI can parse:
   - **Export function** (in the split module): `__wasm_split_00___my_feature___00_export_<hash>_load_feature`
   - **Import function** (in main module): `__wasm_split_00___my_feature___00_import_<hash>_load_feature`
   - **JS load function**: `__wasm_split_load_my_feature_<hash>_load_feature`
4. Generates a `thread_local! { static ...: LazySplitLoader }` for async state
5. Generates the async wrapper that calls `ensure_loaded()` before calling through the import

The double-underscore prefix and `00` delimiters are intentional — they prevent accidental matches with user-defined symbols.

### `#[component(lazy)]`

**File:** `packages/core-macro/src/component.rs`

Wraps a component with a `LazyLoader`:

```rust
#[component(lazy)]
fn AdminPanel(user_id: u32) -> Element { ... }
```

Uses `maybe_wasm_split!` for conditional compilation — on WASM with splitting enabled, the component loads its module on first render; otherwise it calls directly.

### Router Integration

**File:** `packages/router-macro/src/route.rs`

Routes can auto-wrap in lazy loaders, making route-based code splitting automatic.

---

## Layer 2: Binary Splitting CLI

**File:** `packages/wasm-split/wasm-split-cli/src/lib.rs` (1547 lines)

The CLI takes two inputs:

- `original.wasm` — pre-wasm-bindgen binary, preserves relocation data
- `bindgened.wasm` — post-wasm-bindgen binary, the actual deployed module

And runs four phases:

### Phase 1: Discovery

Scans `bindgened.wasm` imports/exports for the `__wasm_split_00___*___00_*` pattern. Extracts:

- Module name
- Hash ID
- Function name / component name

### Phase 2: Call Graph

Parses relocation entries from `original.wasm` (requires `--emit-relocs` during compilation). Walks CODE and DATA section relocations to build a directed call graph. Runs BFS from each split point to find its reachable function set. Accounts for wasm-bindgen transformations (dissolved describe functions).

### Phase 3: Chunk Identification

Functions reachable from multiple split points go into shared chunk modules rather than being duplicated.

### Phase 4: Emission (parallel via rayon)

Produces:

- `main.wasm` — original module with split functions replaced by indirect call stubs and "holes" in the element segments
- `module_N_<name>.wasm` — one file per split point, contains the exported entry function and its unique callees
- `chunk_N_<name>.wasm` — shared code referenced by multiple split modules
- `__wasm_split.js` — JavaScript loader glue

**Main module emission:**

- Replaces split function bodies with stubs (indirect calls through function table)
- Punches holes in element segments where split functions sat
- Re-exports memory, globals, indirect function table for split modules to import
- Runs walrus GC to eliminate unreachable code

**Split module emission:**

- Imports main module's memory and function table
- Imports chunk functions from chunk modules
- Exports only its entry function
- Runs GC

### Build Integration

**File:** `packages/cli/src/build/request.rs` (around line 5105)

The `dx` CLI:

1. Compiles with LTO + debug symbols + `--emit-relocs`
2. Runs wasm-bindgen
3. Invokes wasm-split CLI on both artifacts
4. Writes output files with content-hash suffixes

**Content-hash filename format** (added in commit `95fdca3cb`):

```
module_0_admin-dxh<16_hex_chars>.wasm
```

Hash inputs: file contents + asset options + CLI version, using SipHash. Enables HTTP cache busting while allowing old and new versions to coexist.

---

## Layer 3: JavaScript Loader

**File:** `packages/wasm-split/wasm-split-cli/src/__wasm_split.js`

The generated `__wasm_split.js` uses a `makeLoad(url, deps, fusedImports)` factory:

```javascript
export const __wasm_split_load_my_feature_<hash>_load_feature = makeLoad(
    "/assets/module_0_load_feature-dxh<hash>.wasm",
    [__wasm_split_load_chunk_0],  // wait for dependencies first
    fusedImports
);
```

`makeLoad` returns an async function that:

1. Awaits all dependency loaders (chunks this module imports)
2. Fetches the binary
3. Instantiates via `WebAssembly.instantiateStreaming` with an import object containing:
   - Main module's `memory` (single shared linear memory)
   - Main module's `__indirect_function_table` (shared function pointer table)
   - Main module's `__stack_pointer`, `__tls_base`
   - All previously loaded module exports (from `fusedImports`)
4. Merges the new module's exports into `fusedImports`
5. Calls the callback index through the function table to wake the Rust Future

All modules share one linear memory instance. Split modules fill in the "holes" the main module left in the function table when they load.

---

## Layer 4: Rust Async Runtime

**File:** `packages/wasm-split/wasm-split/src/lib.rs`

### `LazySplitLoader`

Three-state machine:

```rust
enum SplitLoaderState {
    Deferred(LoadFn),     // not started
    Pending,              // JS fetch in flight
    Completed(bool),      // done (success/failure)
}
```

### Future Poll Implementation

```
First poll:
  → Set state to Pending
  → Store waker
  → Call JS load function (passes callback fn ptr + Rc ptr)
  → Return Poll::Pending

Subsequent polls while Pending:
  → Update waker
  → Return Poll::Pending

JS load completes:
  → Calls C callback through function table
  → Callback sets state to Completed
  → Wakes stored waker
  → Next poll returns Poll::Ready(bool)
```

### `LazyLoader<Args, Ret>`

Higher-level wrapper for use in non-component code:

```rust
static LOADER: LazyLoader<Props, Element> =
    lazy_loader!(extern "lazy" fn MyComponent(props: Props) -> Element);

// Load async
let ok = LOADER.load().await;

// Call sync after loading
let result = LOADER.call(args)?;
```

---

## End-to-End Flow

```
Source code with #[wasm_split(module)] annotation
  ↓ macro expansion
FFI symbols: __wasm_split_00___module___00_export/import_<hash>_<fn>
  ↓ rustc (--emit-relocs, LTO, debug=true)
original.wasm + bindgened.wasm
  ↓ wasm-split CLI
main.wasm + module_N.wasm + chunk_N.wasm + __wasm_split.js
  ↓ asset pipeline (content-hash suffix)
main-dxh<hash>.wasm, module_0_fn-dxh<hash>.wasm, ...
  ↓ browser loads main.wasm
App runs; split functions appear as stubs
  ↓ user triggers split boundary (calls annotated async fn)
Rust Future polls LazySplitLoader → Deferred
  ↓ JS makeLoad fetches binary
  ↓ WebAssembly.instantiateStreaming with shared memory/table
  ↓ exports merged into fusedImports
  ↓ callback wakes Rust Future
Future resolves → imported function now in table → execution continues
```

---

## Compilation Requirements

```toml
[profile.release]
lto = true    # required for consistent symbols
debug = true  # required for function name resolution in call graph
```

The build target must be `wasm32-unknown-unknown`. Enable via:

```bash
dx build --wasm-split
```

or in `Dioxus.toml`:

```toml
[release]
wasm-split = true
```

---

## Key Files

| File                                                     | Role                                            |
| -------------------------------------------------------- | ----------------------------------------------- |
| `packages/wasm-split/wasm-split-macro/src/lib.rs`        | `#[wasm_split]`, `lazy_loader!` macros          |
| `packages/wasm-split/wasm-split/src/lib.rs`              | Runtime `LazyLoader`, `LazySplitLoader`, Future |
| `packages/wasm-split/wasm-split-cli/src/lib.rs`          | Binary splitting logic                          |
| `packages/wasm-split/wasm-split-cli/src/__wasm_split.js` | JS module loader template                       |
| `packages/cli/src/build/request.rs`                      | `dx` build integration (~line 5105)             |
| `packages/core-macro/src/component.rs`                   | `#[component(lazy)]`                            |
| `packages/router-macro/src/route.rs`                     | Route-level lazy splitting                      |
| `packages/cli/src/opt/hash.rs`                           | Content-hash filename generation                |
| `notes/architecture/10-WASM-SPLIT.md`                    | Architecture docs                               |

---

## Observations for Resonix

- **Async boundaries are mandatory.** Split points must be `async fn`. There's no way to split at synchronous call sites. This is a fundamental design constraint, not an implementation detail.
- **Static call graph.** All split boundaries are determined at compile time. There's no dynamic/runtime splitting.
- **Shared linear memory.** All modules share one memory instance, so split modules can access any global state from main. This simplifies the model but means memory layout must be stable.
- **Content-hash filenames** enable aggressive HTTP caching — once a chunk is cached it won't be re-fetched until the content changes.
- **Chunk deduplication** is automatic — the CLI identifies shared code and extracts it to shared chunks, so you don't pay for duplicated code even if multiple split points share utilities.
- **The async/await integration is clean** — from Rust's perspective, calling a `#[wasm_split]` function is just `function().await`. The loader machinery is invisible.
