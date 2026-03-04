# Implementation Plan: Multi-Channel Audio Buffers (Approach A)

## Context

Each `ConnectionId` currently maps to a mono `Box<UnsafeCell<[Sample]>>` (block_size samples).
The goal is to allow connections to carry N channels (planar layout: channel `c`, sample `i`
= `data[c * block_size + i]`), enforce channel-count matching at `connect()` time, and expose
a typed `AudioBuffer<'_>` / `AudioBufferMut<'_>` to node `process()` calls. The hot-path
`run()` loop must remain zero-alloc and zero-heavy-compute.

The approach is a zero-cost repr(C) transmute at the process() boundary, exactly mirroring
the existing `NonNull<[Sample]>` → `&[Sample]` transmute.

---

## New / Changed Files

### 1. NEW `primitives/port_descriptor.rs`
```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PortDescriptor {
    pub address: PortAddress,
    pub channels: usize,
}
```
Export from `primitives/mod.rs` and `lib.rs`.

### 2. NEW `primitives/audio_buffer.rs`
Three `repr(C)` types with identical field layout:
```rust
#[repr(C)]
pub(crate) struct RawAudioBuffer {
    pub ptr: NonNull<[Sample]>,   // fat ptr: (pointer, block_size * channels)
    pub channels: usize,
}

#[repr(C)]
pub struct AudioBuffer<'a> {
    ptr: NonNull<[Sample]>,
    channels: usize,
    _phantom: PhantomData<&'a [Sample]>,  // MUST be last
}

#[repr(C)]
pub struct AudioBufferMut<'a> {
    ptr: NonNull<[Sample]>,
    channels: usize,
    _phantom: PhantomData<&'a mut [Sample]>,  // MUST be last
}
```
`Option<RawAudioBuffer>` uses the null-pointer niche on `ptr` (first field = `NonNull`) —
3 words, no discriminant. `Option<AudioBuffer<'_>>` and `Option<AudioBufferMut<'_>>` are
identical. Add a layout test:
```rust
assert_eq!(size_of::<Option<RawAudioBuffer>>(), 3 * size_of::<usize>());
```

`AudioBuffer<'a>` API:
```rust
pub fn channels(&self) -> usize
pub fn channel(&self, c: usize) -> &[Sample]   // data[c*bs .. (c+1)*bs]
pub fn mono(&self) -> &[Sample]                 // panics if channels != 1
// Unsafe constructor (for tests and internal use):
pub unsafe fn from_raw(ptr: NonNull<[Sample]>, channels: usize) -> Self
```

`AudioBufferMut<'a>` mirrors with `channel_mut` and `mono_mut`.

`ptr` and `channels` are `pub(crate)` for internal patching in `run()`.

Export `AudioBuffer` and `AudioBufferMut` from `primitives/mod.rs` and `lib.rs`.

### 3. `primitives/buffer_pool.rs`
Replace `pub type AudioBuffer = Box<UnsafeCell<[Sample]>>` with:
```rust
pub struct ChannelledBuffer {
    pub channels: usize,
    pub data: Box<UnsafeCell<[Sample]>>,  // block_size * channels samples, planar
}

pub struct BufferPool {
    buffers: IntMap<ConnectionId, ChannelledBuffer>,
}
```
Keep `Deref`/`DerefMut` updated to target `IntMap<ConnectionId, ChannelledBuffer>`.

### 4. `traits/describe_ports.rs`
Rename all four methods and change return type:
```rust
pub trait DescribePorts {
    fn input_ports(&self) -> Option<&[PortDescriptor]> { None }
    fn output_ports(&self) -> Option<&[PortDescriptor]> { None }
    fn external_output_ports(&self) -> Option<&[PortDescriptor]> { None }
    fn external_input_ports(&self) -> Option<&[PortDescriptor]> { None }
}
```
Update the blanket `Deref` impl accordingly.

### 5. `errors/graph_connection_error.rs`
Add variant:
```rust
#[error("channel count mismatch: output port has {start_channels} channel(s), input port has {end_channels}")]
ChannelCountMismatch { start_channels: usize, end_channels: usize },
```

### 6. `traits/audio_node.rs`
Change `process()` signature:
```rust
fn process(
    &mut self,
    inputs: &[Option<AudioBuffer<'_>>],
    outputs: &mut [Option<AudioBufferMut<'_>>],
    block_size: BlockSize,
) -> Result<(), AudioNodeRunError>;
```

### 7. `traits/graph.rs`
Update `run()` signature:
```rust
fn run(
    &mut self,
    inputs: &HashMap<ConnectionId, AudioBuffer<'_>>,
    outputs: &mut HashMap<ConnectionId, AudioBufferMut<'_>>,
) -> Result<(), GraphRunError>;
```

### 8. `implementations/graph.rs`

**`Graph` struct** — add one field:
```rust
port_address_to_channel_count: HashMap<PortAddress, usize>,
```
Initialize to `HashMap::new()` in `with_block_size()`.

**`CompiledStep`** — two field type changes only; external slot tuples unchanged:
```rust
input_ptrs: Box<[Option<RawAudioBuffer>]>,
output_ptrs: Box<[Option<RawAudioBuffer>]>,
external_output_slots: Box<[(usize, ConnectionId)]>,  // unchanged
external_input_slots: Box<[(usize, ConnectionId)]>,
```

**`allocate_empty_buffer_for_connection`** — add `channels: usize` param, allocate
`block_size * channels` samples, store `ChannelledBuffer { channels, data: cell_box }`.

**`validate_dense_port_ids` / `count_ports`** — change parameter from
`&[Option<&[PortAddress]>]` to `&[Option<&[PortDescriptor]>]`; extract `.address.port_id()`
inside.

**`register_external_port_addresses`** — change `port_addresses: &[PortAddress]` to
`port_descriptors: &[PortDescriptor]`; use `descriptor.address` everywhere `port_address`
was used.

**`add()`** changes:
1. Call the new `input_ports()` / `output_ports()` / `external_*_ports()` methods.
2. After registering external ports, iterate all four descriptor slices and insert each
   `(descriptor.address → descriptor.channels)` into `port_address_to_channel_count`.

**`connect()`** — insert channel-count validation before `resolve_connection_for_output_port`:
```rust
let start_channels = self.port_address_to_channel_count[&start_port_address];
let end_channels   = self.port_address_to_channel_count[&end_port_address];
if start_channels != end_channels {
    return Err(GraphConnectionError::ChannelCountMismatch { start_channels, end_channels });
}
```

**`resolve_connection_for_output_port`** — look up channel count from
`port_address_to_channel_count`, pass to `allocate_empty_buffer_for_connection(connection_id, channels)`.

**`resolve_audio_buffer_pointers`** — change return to `Box<[Option<RawAudioBuffer>]>`.
For each pool buffer:
```rust
let cb = buffer_pool.get(connection_id)?;
let raw_ptr: *mut [Sample] = cb.data.get();
Some(RawAudioBuffer {
    ptr: NonNull::new_unchecked(raw_ptr),
    channels: cb.channels,
})
```
External slot recording stays `external_slots.push((slot_index, *connection_id))` — channel
count comes from the caller's `AudioBuffer` at `run()` time.

**`run()` signature** — change public API (both trait and impl):
```rust
fn run(
    &mut self,
    inputs: &HashMap<ConnectionId, AudioBuffer<'_>>,
    outputs: &mut HashMap<ConnectionId, AudioBufferMut<'_>>,
) -> Result<(), GraphRunError>;
```

**`run()` external slot patching** — construct `RawAudioBuffer` from caller's typed buffers:
```rust
for &(slot, connection_id) in step.external_output_slots.iter() {
    step.output_ptrs[slot] = outputs.get_mut(&connection_id)
        .map(|buf| RawAudioBuffer { ptr: buf.ptr, channels: buf.channels });
}
for &(slot, connection_id) in step.external_input_slots.iter() {
    step.input_ptrs[slot] = inputs.get(&connection_id)
        .map(|buf| RawAudioBuffer { ptr: buf.ptr, channels: buf.channels });
}
```

**`run()` transmute** — unchanged pattern, updated types:
```rust
let input_buffers: &[Option<AudioBuffer<'_>>] =
    unsafe { transmute(step.input_ptrs.as_ref()) };
let output_buffers: &mut [Option<AudioBufferMut<'_>>] =
    unsafe { transmute(step.output_ptrs.as_mut()) };
```

### 9. Node port descriptor structs (`constant_node.rs`, `multiply_node.rs`, `output_node.rs`)
Change stored arrays from `[PortAddress; N]` to `[PortDescriptor; N]`. All existing ports
declare `channels: 1`. `DescribePorts` impls return `&self.input_port_descriptors` etc.
The public `output_port_address()` helper methods continue to return `PortAddress` (extracted
from the stored `PortDescriptor.address`).

### 10. Node `process()` implementations
All three nodes declare mono ports. Update their `process()` bodies to use
`AudioBuffer::mono()` / `AudioBufferMut::mono_mut()` (equivalent to the previous
`unwrap_or(&[])` / `as_deref_mut()` patterns). Unit tests in each node file must be updated
to construct `AudioBuffer`/`AudioBufferMut` via the unsafe `from_raw` constructor.

---

## Verification

```bash
cd crates/resonix-graph
cargo test
cargo miri test   # verify no Stacked Borrows violations
```

Key test cases to add / update:
- `connect()` with mismatched channel counts returns `ChannelCountMismatch` error
- Single-channel round-trip still passes all existing graph integration tests
- Layout size assertion: `size_of::<Option<RawAudioBuffer>>() == 3 * size_of::<usize>()`
- Existing node unit tests updated to construct `AudioBuffer` / `AudioBufferMut` via `from_raw`
- Existing `run()` call sites updated to pass `HashMap<ConnectionId, AudioBuffer<'_>>`
