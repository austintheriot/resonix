# Resonix WASM Audio Node ABI

This document specifies the interface between the resonix host (`WasmNode`) and a WASM audio module. Both sides must conform exactly for the module to load and run correctly.

---

## Overview

A WASM audio module is a self-contained DSP unit that plugs into the resonix audio graph. The host compiles and instantiates the module, calls a fixed sequence of lifecycle functions to discover its port layout, then drives it in a tight real-time loop. The module owns its own staging buffers inside WASM linear memory; the host copies audio data in and out across the memory boundary each block.

---

## Lifecycle

The host performs the following sequence exactly once at instantiation time, in order:

1. **Compile and instantiate** the WASM module with the host imports described below.
2. **Call `init()`** if the module exports it. The module may call host imports during `init` to cache configuration values and perform one-time allocation. `init` is called before any port queries.
3. **Query port layout** by calling the required port descriptor exports.
4. **Cache buffer pointers** by calling the buffer pointer exports once per port.
5. **Cache `process`** as a typed function handle for the hot path.

At runtime, on every audio block:

1. For each connected input port, copy the host-side audio buffer into WASM linear memory at the cached offset.
2. Call `process(block_size, current_time)`.
3. For each connected output port, copy WASM linear memory at the cached offset back into the host-side audio buffer.

No allocations or export lookups occur during the real-time loop.

---

## Host Imports

The host provides the following functions in the `"resonix"` import namespace. The module may call them at any time, but they are most useful inside `init`.

| Function | Signature | Description |
|---|---|---|
| `get_block_size` | `() -> i32` | Number of samples per block. Fixed for the lifetime of the node. |
| `get_sample_rate` | `() -> i32` | Sample rate in Hz. Fixed for the lifetime of the node. |

In Rust (`wasm32-unknown-unknown`):

```rust
#[link(wasm_import_module = "resonix")]
unsafe extern "C" {
    fn get_block_size() -> i32;
    fn get_sample_rate() -> i32;
}
```

---

## Module Exports

### Optional lifecycle

| Export | Signature | Description |
|---|---|---|
| `init` | `() -> ()` | Called once after instantiation, before any port queries. Use to cache host configuration and set up internal state. Omit if not needed. |

### Required port descriptor queries

These are called once at instantiation time (after `init`). Return values are cached by the host and never re-queried.

| Export | Signature | Description |
|---|---|---|
| `get_input_count` | `() -> i32` | Total number of input ports. |
| `get_output_count` | `() -> i32` | Total number of output ports. |
| `get_input_channel_count` | `(port_id: i32) -> i32` | Channel count for input port `port_id`. |
| `get_output_channel_count` | `(port_id: i32) -> i32` | Channel count for output port `port_id`. |
| `get_input_buffer_ptr` | `(port_id: i32) -> i32` | WASM linear memory byte offset of the input staging buffer for `port_id`. |
| `get_output_buffer_ptr` | `(port_id: i32) -> i32` | WASM linear memory byte offset of the output staging buffer for `port_id`. |

Port IDs are dense and zero-indexed: input ports are `0..get_input_count()`, output ports are `0..get_output_count()`.

**Note on Rust statics:** A Rust `pub static X: i32 = N` compiles to a WASM global whose value is the *linear memory address* of the static — not `N`. Use zero-argument exported functions (as shown above) for all count and pointer queries to avoid this footgun.

### Required hot-path callback

| Export | Signature | Description |
|---|---|---|
| `process` | `(block_size: i32, current_time: f64) -> ()` | DSP callback. Called once per audio block after inputs are written. |

`block_size` is the number of samples in the current block. `current_time` is the stream position in seconds.

---

## Memory Layout

### Staging buffers

Each port has one staging buffer in WASM linear memory. The buffer is **contiguous and planar**: all samples for channel 0 come first, then all samples for channel 1, and so on.

```
buffer_ptr
│← block_size samples (f32) →│← block_size samples (f32) →│ ...
│       channel 0             │       channel 1             │
```

Byte offset of channel `ch` within the buffer:

```
channel_offset = buffer_ptr + ch * block_size * sizeof(f32)
               = buffer_ptr + ch * block_size * 4
```

The host uses the value returned by `get_input_buffer_ptr` / `get_output_buffer_ptr` as `buffer_ptr` and derives channel offsets with this formula. The module must lay out its staging buffers to match.

### Capacity

Staging buffers must be allocated for the maximum block size the module will ever see. Since `block_size` is fixed for the lifetime of the node, modules can query it in `init` and use it as their allocation size. As a safe upper bound, resonix's internal `BlockSize::MAX_BLOCK_SIZE` is 65536 samples.

Each buffer must hold at least `channel_count * block_size * sizeof(f32)` bytes.

### Memory ownership

The module owns its staging buffers. The host never allocates inside WASM linear memory. Buffers must be valid for the entire lifetime of the module — allocate them statically or in `init`, never inside `process`.

---

## Data types

All samples are **`f32` (IEEE 754 single-precision)**. Port IDs, channel counts, and buffer pointers are **`i32`** (WASM's native integer type; pointers are 32-bit offsets into linear memory). `current_time` is **`f64`**.

---

## Invariants and guarantees

| Guarantee | Who provides it |
|---|---|
| `block_size` is constant for the lifetime of the node | Host |
| `sample_rate` is constant for the lifetime of the node | Host |
| `init` is called before any port descriptor query | Host |
| `process` is never called concurrently | Host |
| `process` is not called before instantiation and `init` complete | Host |
| Buffer pointers returned by the ptr queries remain valid for the lifetime of the module | Module |
| Staging buffers have capacity for at least `channel_count * block_size * 4` bytes | Module |
| Port IDs are dense: `0..get_input_count()` and `0..get_output_count()` | Module |
| `get_input_count` / `get_output_count` return the same value on every call | Module |

---

## Error handling

A WASM trap inside any module export propagates to the host as a `WasmNodeCreationError::RuntimeError` (during instantiation) or an `AudioNodeRunError` (during `process`). Traps are not recoverable — the node should be considered invalid after a trap.

---

## Minimal Rust example

```rust
#![no_std]
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }

#[link(wasm_import_module = "resonix")]
unsafe extern "C" {
    fn get_block_size() -> i32;
    fn get_sample_rate() -> i32;
}

const CHANNELS: usize = 2;
const MAX_BLOCK_SIZE: usize = 2048;

static mut INPUT_BUF:  [f32; CHANNELS * MAX_BLOCK_SIZE] = [0.0; CHANNELS * MAX_BLOCK_SIZE];
static mut OUTPUT_BUF: [f32; CHANNELS * MAX_BLOCK_SIZE] = [0.0; CHANNELS * MAX_BLOCK_SIZE];
static mut BLOCK_SIZE: usize = 0;

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    unsafe { BLOCK_SIZE = get_block_size() as usize; }
}

#[unsafe(no_mangle)] pub extern "C" fn get_input_count()  -> i32 { 1 }
#[unsafe(no_mangle)] pub extern "C" fn get_output_count() -> i32 { 1 }

#[unsafe(no_mangle)] pub extern "C" fn get_input_channel_count(_port_id: i32)  -> i32 { CHANNELS as i32 }
#[unsafe(no_mangle)] pub extern "C" fn get_output_channel_count(_port_id: i32) -> i32 { CHANNELS as i32 }

#[unsafe(no_mangle)]
pub extern "C" fn get_input_buffer_ptr(_port_id: i32) -> i32 {
    core::ptr::addr_of!(INPUT_BUF) as i32
}
#[unsafe(no_mangle)]
pub extern "C" fn get_output_buffer_ptr(_port_id: i32) -> i32 {
    core::ptr::addr_of!(OUTPUT_BUF) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn process(block_size: i32, _current_time: f64) {
    let n = block_size as usize;
    for ch in 0..CHANNELS {
        let base = ch * n;
        for i in 0..n {
            unsafe { OUTPUT_BUF[base + i] = INPUT_BUF[base + i] * 0.5; }
        }
    }
}
```

Build with:

```
cargo build --target wasm32-unknown-unknown --release
```

The crate must be `crate-type = ["cdylib"]` and have no `std` dependency.
