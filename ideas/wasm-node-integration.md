# WASM Audio Node Integration

## Background

`WasmNode` wraps a Wasmer `Instance` and must fit into resonix-graph's existing audio pipeline:

- `process()` is called from a **pre-compiled flat execution plan** — no allocation, no topology changes, hot path.
- Buffers are raw-pointer slices (`&[Option<&[Sample]>]` / `&mut [Option<&mut [Sample]>]`) with SRW provenance from `UnsafeCell`.
- Port IDs are **dense 0..N** (used as direct slice indices).
- Everything lives in `no_std` + `alloc`; no `std`.

The central challenge: WASM runs in its own linear memory. Rust audio buffers live in the host's memory. These two address spaces are **disjoint**, so we must decide how to cross them on every `process()` call.

---

## What the WASM Module Must Export

Regardless of approach, we need a **convention** for what a WASM audio module exposes. Three values are needed at instantiation time:

| Export                                                                    | Type   | Purpose                                                           |
| ------------------------------------------------------------------------- | ------ | ----------------------------------------------------------------- |
| `resonix_input_count() -> i32`                                            | `func` | Number of input ports                                             |
| `resonix_output_count() -> i32`                                           | `func` | Number of output ports                                            |
| `get_input_channel_count(port_id: i32) -> i32`                   | `func` | Channel count for input port                                      |
| `get_output_channel_count(port_id: i32) -> i32`                  | `func` | Channel count for output port                                     |
| `get_input_buffer_ptr(port_id: i32, ch: i32) -> i32`     | `func` | WASM byte offset for input port's per-channel staging buffer      |
| `get_output_buffer_ptr(port_id: i32, ch: i32) -> i32`    | `func` | WASM byte offset for output port's per-channel staging buffer     |
| `resonix_process(block_size: i32, current_time: f64)`                    | `func` | Main DSP callback                                                 |

All counts and pointers are queried at `new()` time and cached. Port descriptors are populated from the count/channel queries. The graph's dense-port invariant is satisfied automatically since ports 0..input_count and 0..output_count are contiguous.

**Per-channel buffer pointers** (rather than a per-port base + stride) let the WASM module lay out its memory however it wants — the host doesn't need to know the inter-channel stride. Each `(port, channel)` offset is queried once at instantiation and stored in `Box<[u64]>`.

**Note on Rust `static` globals vs WASM globals:** A Rust `pub static X: i32 = N` compiles to a WASM global whose value is the *address* of the static in linear memory, not the constant value. Use exported functions (zero-arg, returning i32) for count queries to avoid this footgun.

---

## Integration Options

### Option A — Copy-In / Copy-Out via WASM Linear Memory

**How it works:**

At `new()`, for each port call `resonix_get_input/output_channel_count(port)` and then `resonix_get_input/output_channel_buffer_ptr(port, ch)` for every channel. Cache the resulting byte offsets in `Box<[u64]>` per port. At `process()`:

1. For each connected input port, for each channel: `MemoryView::write(cached_offset, samples_as_bytes)`.
2. Call `resonix_process(block_size, current_time)`.
3. For each connected output port, for each channel: `MemoryView::read(cached_offset, out_bytes)`.

The WASM module owns its staging buffers (allocated at startup, sized for `max_block_size`). The host copies exactly `block_size * 4` bytes per channel per port — it doesn't need to know the stride between channels since each channel has its own cached pointer.

**Pros:**

- Simple, no unsafe WASM memory tricks on the host.
- WASM module is self-contained — manages its own memory layout.
- Compatible with Wasmer's `Memory::view()` API across all targets (native + js-default).
- No cross-boundary aliasing concerns.

**Cons:**

- Two full memcpy passes per block (one in, one out) regardless of whether a port is connected.
- At 512 samples × 2 ch × 4 bytes × 2 copies ≈ 8 KiB/block — acceptable but not free.
- WASM module must reserve a fixed staging area large enough for the worst-case block size.

**Verdict:** Simplest correct approach. Good default.

---

### Option B — Zero-Copy via Shared Memory / WASM-Threads

**How it works:**

Use `SharedArrayBuffer` (web) or POSIX shared memory (native) to map a single physical memory region visible to both host and WASM. WASM linear memory is backed by the shared region. Host writes input slices directly into the shared region; WASM reads and writes in place; host reads outputs directly.

**Pros:**

- True zero-copy: no memcpy on hot path.
- Ideal for very large block sizes or high channel counts.

**Cons:**

- Requires `SharedArrayBuffer` + COOP/COEP headers on web — restricts deployment contexts.
- Wasmer's `js-default` feature has limited shared memory support; native Wasmer requires the `threads` feature and WASM modules compiled with `-pthread`.
- WASM module must be compiled with threading support (`wasm32-unknown-unknown` + atomics).
- Adds substantial complexity to both the host integration and the module authoring experience.
- Aliasing between host pointers and WASM linear memory would require careful synchronization (atomics or sequencing guarantees).

**Verdict:** High complexity, deployment constraints. Worthwhile only if profiling shows copy-in/copy-out is a measurable bottleneck.

---

### Option C — Host-Provided Function Imports (Callback Model)

**How it works:**

Instead of staging buffers in WASM memory, the WASM module imports host functions:

```wasm
(import "resonix" "read_input"  (func (param i32 i32 i32) (result f32)))
;;                                         port  sample  channel
(import "resonix" "write_output" (func (param i32 i32 i32 f32)))
```

The WASM module calls these per-sample. The host `WasmImporter` implementation reads/writes the host-side audio buffers directly.

**Pros:**

- No WASM linear memory management at all on the host side.
- WASM module is thin — just algorithm logic, no memory boilerplate.

**Cons:**

- One Wasmer host-function call per sample per channel per port. At 48 kHz, 512-sample blocks, stereo: ~200k FFI calls/block. FFI overhead dominates completely.
- Wasmer's native call overhead is ~10–50 ns/call; at 200k calls that is 2–10 ms — unacceptable for real-time audio.
- Debugging is difficult; stack traces cross language boundaries at every sample.

**Verdict:** Unusable for audio. Rejected.

---

### Option D — WASM-Side Buffer Ownership with Pointer-Pass ABI

**How it works:**

WASM module exports an allocator (`resonix_alloc(size) -> i32`) and the host uses it to allocate I/O buffers _inside_ WASM linear memory at `new()` time. At `process()`:

1. Host writes input data into the pre-allocated WASM-side buffers via `Memory::view()`.
2. Calls `resonix_process(in_ptrs_ptr, out_ptrs_ptr, block_size)` — WASM receives raw WASM-linear pointers.
3. Host reads output data from the WASM-side buffers via `Memory::view()`.

This is essentially Option A but lets the WASM module decide where to place its buffers.

**Pros:**

- WASM module can place buffers in a region that suits its internal data layout.
- Module controls its own memory lifecycle.
- No fixed staging-area convention needed.

**Cons:**

- Requires WASM module to export a compatible allocator and a pointer-ABI `process` signature.
- More complex module authoring (authors must expose an allocator).
- Essentially same copy count as Option A, with added complexity.
- WASM allocator adds module binary size.

**Verdict:** More complex than Option A for equivalent throughput. Use only if modules need to control their own buffer placement.

---

## Recommendation

**Implement Option A (Copy-In / Copy-Out) as the baseline.**

- Correctness is straightforward.
- Memcpy cost is bounded and predictable; at typical block sizes it is negligible compared to DSP computation.
- Module authoring is simple: export the seven functions listed above.
- No new Wasmer features needed beyond what is already configured.
- Option B can be layered on later if benchmarks show copy cost is a bottleneck.

---

## Implementation Sketch

### Module-Side Convention (WASM)

```wasm
;; counts — zero-arg functions (NOT globals: Rust statics export as address globals, not value globals)
(export "resonix_input_count"  (func))  ;; () -> i32
(export "resonix_output_count" (func))  ;; () -> i32

;; per-port queries, called once per (port, channel) at instantiation time
(export "get_input_channel_count"         (func))  ;; (port_id: i32) -> i32
(export "get_output_channel_count"        (func))  ;; (port_id: i32) -> i32
(export "get_input_buffer_ptr"    (func))  ;; (port_id: i32, ch: i32) -> i32
(export "get_output_buffer_ptr"   (func))  ;; (port_id: i32, ch: i32) -> i32

;; hot-path callback
(export "resonix_process" (func))  ;; (block_size: i32, current_time: f64) -> ()
```

Each channel buffer must be sized for `MAX_BLOCK_SIZE * sizeof(f32)` bytes. The host copies exactly `block_size * 4` bytes per channel per call — it doesn't need to know the stride between channels.

### Host-Side `WasmNode`

```rust
struct PortInfo {
    channel_count: usize,
    channel_offsets: Box<[u64]>,  // one WASM byte offset per channel
}

pub struct WasmNode {
    node_id: NodeId,
    store: Store,
    instance: Instance,
    port_descriptors: WasmNodePortDescriptors,
    input_ports: Box<[PortInfo]>,
    output_ports: Box<[PortInfo]>,
    process_fn: TypedFunction<(i32, f64), ()>,
}
```

**`new()` steps:**

1. Compile and instantiate module.
2. Call `resonix_input_count()` / `resonix_output_count()` → get port counts.
3. For each input port `p`: call `get_input_channel_count(p)` → `channel_count`; then for each channel `ch` call `get_input_buffer_ptr(p, ch)` → cache `PortInfo { channel_count, channel_offsets }`.
4. Same for output ports using `get_output_channel_count` / `get_output_buffer_ptr`.
5. Build `WasmNodePortDescriptors` from the collected port infos.
6. Cache `process_fn` as `TypedFunction<(i32, f64), ()>`.

**`process()` steps:**

1. Scope 1: borrow `instance` to get `Memory`, create `MemoryView`. For each connected input port, for each channel: `view.write(channel_offsets[ch], samples_as_bytes)`. Drop memory borrow.
2. Call `process_fn.call(&mut self.store, block_size, current_time)`.
3. Scope 2: borrow `instance` again for `Memory`. For each connected output port, for each channel: `view.read(channel_offsets[ch], out_bytes)`.

The two-scope pattern is required because `instance.exports.get_memory()` borrows `self.instance`, which conflicts with the `&mut self.store` needed by `process_fn.call`. Scoping the memory borrow ends it before the call.

---

## Risks and Open Questions

| Issue                    | Notes                                                                                                                                                                                                |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `block_size` variability | WASM staging buffers are sized for `MAX_BLOCK_SIZE` by the module. If `block_size > MAX_BLOCK_SIZE` at runtime, WASM-side OOB access will trap. Module authors must size buffers appropriately.      |
| `Memory::view()` cost    | Wasmer acquires a new `MemoryView` on each call. The cost is a single pointer read; negligible in practice but worth profiling if many ports/channels.                                               |
| Error propagation        | WASM traps map to `RuntimeError`. Converted to `AudioNodeRunError::Wasm(String)` at the boundary.                                                                                                   |
| WASM module discovery    | How do users supply WASM bytes? File path, embedded bytes, URL? Out of scope for this layer but affects the `new()` API.                                                                             |
| `no_std` Wasmer          | Wasmer itself requires `std`. `WasmNode` cannot be used in a pure `no_std` context — acceptable since `wasmer` is already a dependency.                                                             |
