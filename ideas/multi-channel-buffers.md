# Multi-Channel Audio Buffers: Design Proposals

## Context

Currently, each `ConnectionId` maps to a `Box<UnsafeCell<[Sample]>>` of exactly `block_size` samples — a mono signal. The compiled plan (`Vec<CompiledStep>`) holds `Box<[Option<NonNull<[Sample]>>]>` per step, transmuted zero-cost into `&[Option<&[Sample]>]` for node `process()` calls.

The goal is to allow connections to carry N channels (e.g., stereo, surround) while preserving:
- Zero allocations in `run()`
- Zero heavy computation in `run()` (minimal patching/lookups only)
- The existing sound unsafe model (UnsafeCell + SRW provenance + topological ordering)

---

## Shared Assumptions

Across all approaches, channel count is a property of the **port** (declared by the node's port descriptor). It is validated at `connect()` time and baked into the compiled plan. The channel count of a connection is therefore:
- Statically known per `ConnectionId` at compile-plan time
- Never changes without a topology change (which invalidates the plan)
- The **source** port's declared channel count is authoritative; the graph rejects connections where source and destination disagree (or optionally inserts a format-conversion node)

---

## Buffer Pool Change (shared by all approaches)

The buffer pool currently stores `Box<UnsafeCell<[Sample]>>`. The natural extension is a
wrapper struct that pairs the channel count with the existing allocation:

```rust
struct ChannelledBuffer {
    channels: usize,
    data: Box<UnsafeCell<[Sample]>>,  // block_size * channels samples, planar layout
}

// BufferPool becomes:
IntMap<ConnectionId, ChannelledBuffer>
```

`channels` lives outside `UnsafeCell` because it is immutable after allocation — no interior
mutability is needed for it.

**Why not `UnsafeCell<ChannelledBuffer(Box<[Sample]>)>`?**

`UnsafeCell`'s SRW provenance covers the memory of the cell itself, not memory that the
cell's contents *point to* through a separate allocation. With
`UnsafeCell<ChannelledBuffer>`:

- `UnsafeCell::get()` gives `*mut ChannelledBuffer` — SRW provenance over the struct bytes
  (i.e., the `Box` pointer field stored inline)
- The actual sample data lives in a **separate heap allocation** owned by the `Box`, which
  carries Unique provenance
- Pointers derived by reading through the `Box` to the sample data inherit Unique provenance,
  not SRW

This is the same aliasing problem the current `Box<UnsafeCell<[Sample]>>` design solves: by
wrapping the sample data *directly* in `UnsafeCell` (one allocation), `UnsafeCell::get()`
gives SRW provenance on the data itself, so the compiled plan can hold simultaneous input
and output pointers to the same buffer without either invalidating the other.

Buffer allocation changes from `block_size` samples to `block_size * channels` samples.
All other UnsafeCell/SRW-provenance reasoning is unchanged.

**Buffer layout**: planar — channel `c`, sample `i` = `data[c * block_size + i]`.
This allows contiguous per-channel slices (`channel(c) → &data[c*bs..(c+1)*bs]`), unlike
interleaved layout which requires strided access.

---

## Approach A: repr(C) Transmute at process() Boundary

### Core idea

Store a named raw struct carrying `(ptr, len, channels)` in `CompiledStep`. Transmute it
zero-cost to a lifetime-bearing `AudioBuffer<'_>` at the `process()` call boundary, exactly
as the current design transmutes `NonNull<[Sample]>` to `&[Sample]` today.

### New types

```rust
/// Stored in CompiledStep.
#[repr(C)]
struct RawAudioBuffer {
    ptr: NonNull<[Sample]>,  // fat ptr (pointer + length = block_size * channels); SRW provenance
    channels: usize,
}

/// Passed to process() as inputs.
#[repr(C)]
pub struct AudioBuffer<'a> {
    ptr: NonNull<[Sample]>,
    channels: usize,
    _phantom: PhantomData<&'a [Sample]>,
}

/// Passed to process() as outputs.
#[repr(C)]
pub struct AudioBufferMut<'a> {
    ptr: NonNull<[Sample]>,
    channels: usize,
    _phantom: PhantomData<&'a mut [Sample]>,
}
```

`Option<RawAudioBuffer>` uses the null pointer niche on the pointer component of
`ptr: NonNull<[Sample]>` (first field). Layout is 3 words with no discriminant.
`Option<AudioBuffer<'_>>` has identical layout. `PhantomData` must remain the last field.

### CompiledStep

```rust
struct CompiledStep {
    node: *mut dyn AudioNode,
    input_ptrs: Box<[Option<RawAudioBuffer>]>,
    output_ptrs: Box<[Option<RawAudioBuffer>]>,
    external_output_slots: Box<[(usize, ConnectionId)]>,
    external_input_slots: Box<[(usize, ConnectionId)]>,
    block_size: BlockSize,
}
```

### run() hot path

```rust
let input_buffers: &[Option<AudioBuffer<'_>>] =
    unsafe { transmute(step.input_ptrs.as_ref()) };
let output_buffers: &mut [Option<AudioBufferMut<'_>>] =
    unsafe { transmute(step.output_ptrs.as_mut()) };
```

Zero-cost: `RawAudioBuffer` and `AudioBuffer` are `repr(C)` with identical field order.
`Option` niche layout is identical. Only the provenance annotation (phantom lifetime) differs.

### Soundness

- **repr(C) stability**: the transmute is only sound if layout is guaranteed. Enforce with
  `#[repr(C)]` and a test asserting
  `size_of::<Option<RawAudioBuffer>>() == 3 * size_of::<usize>()`. A layout regression is
  silent and unsafe.
- **SRW provenance**: unchanged from today; pointer comes from `ChannelledBuffer::data`'s
  `UnsafeCell::get()`.
- **No aliased mutation**: topological order ensures upstream writes before downstream reads;
  planar sub-slices within one connection do not overlap; separate connections have separate
  allocations.

### Pros / Cons

**Pros**
- Hot path remains zero-cost (same transmute pattern as today, just 3 words instead of 2)
- No per-step stack allocation or construction overhead

**Cons**
- `repr(C)` layout contract must be maintained; regressions are silent
- `PhantomData` placement discipline required (must be last field)
- More unsafe invariants to document

---

## Approach B: Explicit Construction at process() Boundary

### Core idea

Store `NonNull<[Sample]>` and channel count as separate parallel arrays in `CompiledStep`
(unchanged pointer type from today). In `run()`, iterate over both arrays and construct
`AudioBuffer` structs — no `repr(C)`, no transmute.

### CompiledStep

```rust
struct CompiledStep {
    node: *mut dyn AudioNode,
    input_ptrs: Box<[Option<NonNull<[Sample]>>]>,     // unchanged from today
    output_ptrs: Box<[Option<NonNull<[Sample]>>]>,
    input_channel_counts: Box<[usize]>,               // parallel to input_ptrs
    output_channel_counts: Box<[usize]>,
    external_output_slots: Box<[(usize, ConnectionId)]>,
    external_input_slots: Box<[(usize, ConnectionId)]>,
    block_size: BlockSize,
}
```

### run() hot path

`process()` takes `&[Option<AudioBuffer<'_>>]` — a contiguous slice. To avoid allocation,
the compiled step pre-allocates scratch boxes filled each call:

```rust
// In CompiledStep (allocated at plan-compile time, not in run()):
input_scratch: Box<[Option<AudioBuffer<'static>>]>,
output_scratch: Box<[Option<AudioBufferMut<'static>>]>,
```

In `run()`:

```rust
for (scratch, (ptr, ch)) in step.input_scratch.iter_mut()
    .zip(step.input_ptrs.iter().zip(step.input_channel_counts.iter()))
{
    *scratch = ptr.map(|p| AudioBuffer {
        ptr: p,
        channels: *ch,
    });
}
// transmute 'static → actual lifetime at call site
let input_buffers: &[Option<AudioBuffer<'_>>] =
    unsafe { transmute(step.input_scratch.as_ref()) };
```

Only a lifetime transmute at the call site — the struct construction happens via
explicit field writes into pre-allocated memory, requiring no layout contract between
`AudioBuffer` and any raw storage type.

Alternatively, if `AudioBuffer` is defined without a lifetime (an explicitly unsafe type),
the scratch boxes can be used directly without any transmute.

### Soundness

- No layout contract between stored type and process type — struct fields are written
  explicitly, not reinterpreted
- Lifetime transmute (`'static` → `'_`) is the same pattern used elsewhere in the graph
  for node pointers
- N writes per step where N = port count (typically 2–8); pre-allocated scratch means no
  allocation in `run()`

### Pros / Cons

**Pros**
- No `repr(C)` requirement; no layout brittleness
- Struct construction is obviously correct
- Easier to reason about for future maintainers

**Cons**
- N field writes per step into scratch memory (vs. zero writes with transmute)
- Two extra `Box` allocations per compiled step (input and output scratch)
- `'static` lifetime in stored scratch type requires a transmute at the call site anyway
  (though only of the lifetime, not the layout)

---

## Approach C: Pre-split Channel Pointers in CompiledStep

### Core idea

Keep the single flat planar allocation, but at plan-compile time, pre-compute one
`NonNull<[Sample]>` per channel per port. The hot path transmutes inner per-channel slices.

### CompiledStep

```rust
struct CompiledStep {
    node: *mut dyn AudioNode,
    input_ptrs: Box<[Option<Box<[NonNull<[Sample]>]>>]>,  // per-port → per-channel
    output_ptrs: Box<[Option<Box<[NonNull<[Sample]>]>>]>,
    ...
}
```

### Pros / Cons

**Pros**
- `process()` receives clean per-channel slices with no arithmetic

**Cons**
- One extra `Box` allocation per port at plan-compile time
- Two levels of pointer indirection in hot path
- Transmuting nested `Box`/`&` is harder to justify soundly; inner `Box` has Unique
  provenance, not SRW — requires additional argument
- `&mut [&mut [Sample]]` construction is awkward

---

## Approach D: Global Channel Count on the Graph

### Core idea

Single `channels: usize` on the `Graph`. All buffers use this count. Buffer layout becomes
`block_size * channels` flat samples. The `process()` signature receives `channels` as a
parameter.

### Pros / Cons

**Pros**
- Smallest possible change to existing code and unsafe surface

**Cons**
- Cannot mix mono control signals with stereo audio in the same graph — a fundamental DSP
  requirement
- No type enforcement of layout convention (interleaved vs. planar)

---

## Recommendation

Use the **buffer pool wrapper** described above (`ChannelledBuffer { channels, data }`)
for all approaches — this is the only sound way to associate channel count with the existing
`Box<UnsafeCell<[Sample]>>` allocation.

Between Approach A and B for the `process()` boundary:

- **Approach A** (repr(C) transmute) matches the existing design philosophy and has zero
  hot-path overhead. Prefer this if the layout discipline can be enforced via tests and
  documented invariants.
- **Approach B** (explicit construction) eliminates all layout contracts at the cost of N
  field writes per step. Prefer this if the codebase grows in complexity and the repr(C)
  invariant becomes hard to maintain.

The required changes for either path:
1. Add `channels: usize` to port descriptor types and `Connection`
2. Introduce `ChannelledBuffer` wrapper in `BufferPool`
3. Change buffer allocation to `block_size * channels` samples (planar)
4. Update `compile()` to extract and store channel counts per port slot
5. Update external slot patching in `run()`
6. Change `AudioNode::process` signature to use `AudioBuffer<'_>` / `AudioBufferMut<'_>`
7. Update all existing node implementations
