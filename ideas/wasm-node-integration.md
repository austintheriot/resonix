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

| Export | Type | Purpose |
|---|---|---|
| `resonix_input_count` | `i32` (global or func) | Number of input ports |
| `resonix_output_count` | `i32` (global or func) | Number of output ports |
| `resonix_process` | `func` | Main DSP callback |
| `resonix_get_buffer_ptr` | `func -> i32` | Returns offset into WASM linear memory for I/O staging area |

Port descriptors are populated at `new()` time by reading these exports. The graph's dense-port invariant is satisfied automatically since ports 0..input_count and 0..output_count are contiguous.

---

## Integration Options

### Option A — Copy-In / Copy-Out via WASM Linear Memory

**How it works:**

At `new()`, query `resonix_get_buffer_ptr` once and cache the offset. At `process()`:

1. Copy each input buffer from host memory → WASM linear memory at cached offsets.
2. Call `resonix_process(block_size)`.
3. Copy each output slice from WASM linear memory → host output buffers.

The WASM module owns a fixed staging region (allocated at WASM startup, size = `max_block_size * (input_count + output_count) * sizeof(f32)`). Layout is agreed by convention: inputs first, then outputs, interleaved by channel.

```
[in0_ch0 ... in0_chN | in1_ch0 ... | out0_ch0 ... | out1_ch0 ...]
```

The host-side `WasmNode` pre-computes byte offsets for each port at `new()` time and stores them as `Box<[usize]>`.

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

WASM module exports an allocator (`resonix_alloc(size) -> i32`) and the host uses it to allocate I/O buffers *inside* WASM linear memory at `new()` time. At `process()`:

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
- Module authoring is simple: export `resonix_input_count`, `resonix_output_count`, `resonix_get_buffer_ptr`, `resonix_process`.
- No new Wasmer features needed beyond what is already configured.
- Option B can be layered on later if benchmarks show copy cost is a bottleneck.

---

## Implementation Sketch

### Module-Side Convention (WASM)

```wasm
;; exports expected by the host
(export "resonix_input_count"  (global i32))   ;; or func returning i32
(export "resonix_output_count" (global i32))
(export "resonix_get_buffer_ptr" (func (result i32)))
(export "resonix_process" (func (param i32)))  ;; param = block_size
```

Buffer layout inside WASM linear memory (from `resonix_get_buffer_ptr`):
```
offset + 0                     : input  port 0 samples [f32 × block_size]
offset + block_size*4          : input  port 1 samples
...
offset + input_count*block_size*4  : output port 0 samples
...
```

### Host-Side `WasmNode`

```rust
pub struct WasmNode {
    node_id: NodeId,
    store: Store,
    instance: Instance,
    port_descriptors: WasmNodePortDescriptors,

    // Pre-computed at new(): byte offsets into WASM linear memory for each port
    input_offsets: Box<[u64]>,   // len = input_count
    output_offsets: Box<[u64]>,  // len = output_count

    // Cached function handles (avoids repeated export lookup on hot path)
    process_fn: TypedFunction<i32, ()>,
}
```

**`new()` steps:**
1. Compile and instantiate module.
2. Read `resonix_input_count` / `resonix_output_count` globals/funcs → build `PortDescriptor` arrays.
3. Call `resonix_get_buffer_ptr()` → get base byte offset.
4. Compute and cache `input_offsets` and `output_offsets` from base + port index × block_size × 4.
5. Cache `process_fn` as a `TypedFunction<i32, ()>`.

**`process()` steps:**
1. Get `Memory` view from `instance`.
2. For each connected input port: `memory.view().write(offset, sample_bytes)`.
3. Call `process_fn.call(&mut store, block_size as i32)`.
4. For each connected output port: `memory.view().read(offset, out_slice)`.

**Open question:** `Store` must be mutably borrowed to call functions, but `WasmNode` must implement `&mut self` on `process()`. Store can live inside `WasmNode` since the node has exclusive `&mut self` access during `process()`. No borrow conflicts.

---

## Risks and Open Questions

| Issue | Notes |
|---|---|
| `block_size` variability | WASM staging buffer is sized at `new()`. If block_size changes at runtime the buffer may be too small. Either cap at a max block size or re-allocate staging at process time (one-time realloc on change). |
| Multi-channel buffers | Current process signature uses `&[Sample]` slices per port, not per channel. If ports are multi-channel, need a layout decision (interleaved vs planar). |
| `Memory::view()` cost | Wasmer's `MemoryView` acquires a reference on each call. Cache the memory handle if the API allows, or measure whether it matters. |
| Error propagation | WASM traps (divide by zero, OOB memory) must be caught and converted to `AudioNodeRunError`. `TypedFunction::call` returns `Result<_, RuntimeError>`. |
| WASM module discovery | How do users supply WASM bytes? File path, embedded bytes, URL? Out of scope for this layer but affects the `new()` API. |
| `no_std` Wasmer | Wasmer itself is `std`. `WasmNode` therefore cannot be used in a pure `no_std` context. This is acceptable — it already imports `wasmer` which requires std. |
