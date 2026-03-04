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

## Approach A: RawAudioBuffer Transmute (recommended)

### Core idea

Keep the single flat allocation per connection, but increase it to `block_size * channels` samples using **planar** layout (channel `c`, sample `i` = `data[c * block_size + i]`). Replace the `NonNull<[Sample]>` fat pointer in `CompiledStep` with a 3-word struct that also carries the channel count, laid out identically for zero-cost transmute.

### Buffer Pool

Unchanged type: `IntMap<ConnectionId, Box<UnsafeCell<[Sample]>>>`.
Allocation changes from `block_size` samples to `block_size * channels` samples.
All UnsafeCell/SRW-provenance reasoning remains identical.

### New types

```rust
/// Stored in CompiledStep — same repr as AudioBuffer / AudioBufferMut.
#[repr(C)]
struct RawAudioBuffer {
    ptr: NonNull<Sample>,  // SRW provenance from UnsafeCell::get()
    len: usize,            // = block_size * channels
    channels: usize,
}

/// Passed to process() as inputs (shared borrow).
#[repr(C)]
pub struct AudioBuffer<'a> {
    ptr: NonNull<Sample>,
    len: usize,
    channels: usize,
    _phantom: PhantomData<&'a Sample>,
}

/// Passed to process() as outputs (exclusive borrow).
#[repr(C)]
pub struct AudioBufferMut<'a> {
    ptr: NonNull<Sample>,
    len: usize,
    channels: usize,
    _phantom: PhantomData<&'a mut Sample>,
}
```

`Option<RawAudioBuffer>` uses the null niche on `ptr: NonNull<Sample>` (first field, same as `Option<NonNull<T>>`). The layout is 3 words with no discriminant. `Option<AudioBuffer<'_>>` and `Option<AudioBufferMut<'_>>` have identical layout. The `PhantomData` is zero-sized and placed after all data fields.

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

The transmute is zero-cost: `RawAudioBuffer` and `AudioBuffer` are `repr(C)` with identical field order and types. `Option` niche layout is the same. Only the provenance annotation (phantom lifetime) changes.

External slot patching constructs `RawAudioBuffer` from the caller-supplied buffer and the connection's channel count (stored alongside `external_output_slots`/`external_input_slots` in the compiled step):

```rust
for &(slot, connection_id, channels) in step.external_output_slots.iter() {
    step.output_ptrs[slot] = outputs.get_mut(&connection_id).map(|buf| {
        // buf: AudioBufferMut<'_> from caller — has ptr, len, channels
        RawAudioBuffer { ptr: buf.ptr, len: buf.len, channels }
    });
}
```

No allocations; the 3-word struct is constructed directly into the boxed slice slot.

### process() signature

```rust
fn process(
    &mut self,
    inputs: &[Option<AudioBuffer<'_>>],
    outputs: &mut [Option<AudioBufferMut<'_>>],
    block_size: BlockSize,
) -> Result<(), AudioNodeRunError>;
```

`AudioBuffer` provides ergonomic channel access:

```rust
impl<'a> AudioBuffer<'a> {
    pub fn channels(&self) -> usize { self.channels }
    pub fn channel(&self, c: usize) -> &[Sample] {
        let bs = self.len / self.channels;
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr().add(c * bs), bs) }
    }
    // Convenience for mono: panics if channels != 1
    pub fn mono(&self) -> &[Sample] { self.channel(0) }
}
```

### Soundness

- **UnsafeCell + SRW provenance**: same as today; the base pointer comes from `UnsafeCell::get()` and is widened to `NonNull<Sample>` (thin ptr). SRW provenance is preserved; Unique retags inside `process()` cannot invalidate it.
- **No aliased mutation**: topological order still ensures upstream writes before downstream reads; channel layout is planar so sub-slices within one connection don't overlap; separate connections have separate allocations.
- **Repr(C) stability**: the transmute is only sound if the layout guarantee is upheld. This must be enforced with `#[repr(C)]`, a test asserting `size_of::<Option<RawAudioBuffer>>() == 3 * size_of::<usize>()`, and no reordering of fields.
- **MIRI**: the SRW-provenance argument is the same as the existing implementation; no new aliasing patterns are introduced.

### Pros / Cons

**Pros**
- Hot path remains zero-cost transmute (same pattern as today)
- Single contiguous allocation per connection (no extra per-channel allocations)
- Channel count lives alongside the pointer — no separate lookup
- Minimal structural change to `CompiledStep`, `BufferPool`, `run()`
- Clean `AudioBuffer` API for node authors; mono nodes unchanged in spirit
- External run() API can stay as-is or upgrade symmetrically

**Cons**
- `repr(C)` layout contract must be maintained across refactors; a layout regression is silent and unsafe
- `PhantomData` placement discipline required (must be last field)
- More unsafe invariants to document and test
- Nodes that previously received `&[Sample]` need updating (no backwards compat, acceptable per spec)

---

## Approach B: Pre-split Channel Pointers in CompiledStep

### Core idea

Keep the single flat planar allocation, but at plan-compile time, pre-compute one `NonNull<[Sample]>` per **channel** per port. Store these as a nested structure in `CompiledStep`. The hot path transmutes the inner per-channel pointer array.

### CompiledStep

```rust
struct CompiledStep {
    node: *mut dyn AudioNode,
    // Per-port: a boxed slice of per-channel fat pointers
    input_ptrs: Box<[Option<Box<[NonNull<[Sample]>]>>]>,
    output_ptrs: Box<[Option<Box<[NonNull<[Sample]>]>>]>,
    ...
}
```

### run() hot path

For each step, for each connected port:
```rust
// transmute Box<[NonNull<[Sample]>]> → &[&[Sample]] (inputs)
// transmute Box<[NonNull<[Sample]>]> → &mut [&mut [Sample]] (outputs)
```

But the outer slice (`Box<[Option<Box<...>>]>`) can't be zero-cost transmuted to `&[Option<&[&[Sample]]>]` because the inner `Box` has Unique provenance, not SRO/SRW. A second transmute level adds soundness risk.

### process() signature

```rust
fn process(
    &mut self,
    inputs: &[Option<&[&[Sample]]>],
    outputs: &mut [Option<&mut [&mut [Sample]]>],
    block_size: BlockSize,
) -> Result<(), AudioNodeRunError>;
```

### Pros / Cons

**Pros**
- `process()` receives clean per-channel slices with no arithmetic needed
- Channel count implicit from slice length

**Cons**
- One extra `Box` allocation per port per node at plan-compile time (more total allocations)
- Two levels of pointer indirection in the hot path (outer per-port slice, inner per-channel slice)
- Transmuting nested `Box`/`&` is more complex and harder to reason about soundly; the outer `Box` → `&` transmute at the slice-of-Options level is not covered by the existing SRW argument
- `&mut [&mut [Sample]]` references are awkward to construct soundly (need raw-ptr intermediaries to avoid re-borrow conflicts)

---

## Approach C: Stack-Constructed AudioBuffer in run()

### Core idea

Store the flat pointer and a separate channel count in `CompiledStep`, and construct a lightweight `AudioBuffer` struct on the stack inside the `run()` loop — no transmute required.

### CompiledStep

```rust
struct CompiledStep {
    node: *mut dyn AudioNode,
    input_ptrs: Box<[Option<NonNull<[Sample]>>]>,    // as today
    output_ptrs: Box<[Option<NonNull<[Sample]>>]>,
    input_channel_counts: Box<[usize]>,              // NEW: parallel to input_ptrs
    output_channel_counts: Box<[usize]>,
    ...
}
```

### run() hot path

```rust
let mut input_buffers: [Option<AudioBuffer<'_>>; MAX_PORTS] = [None; MAX_PORTS];
for i in 0..step.input_ptrs.len() {
    if let Some(ptr) = step.input_ptrs[i] {
        let data = unsafe { &*ptr.as_ptr() };
        input_buffers[i] = Some(AudioBuffer { data, channels: step.input_channel_counts[i] });
    }
}
(&mut *step.node).process(&input_buffers[..step.input_ptrs.len()], ...)?;
```

### Pros / Cons

**Pros**
- No new unsafe layout contracts; no transmute of new types
- Conceptually simple — struct construction is obviously correct
- `CompiledStep` changes are minimal

**Cons**
- Stack array of size `MAX_PORTS` is zeroed on every `run()` step call; with `MAX_PORTS = 64` and 3-word `Option<AudioBuffer>`, this is ~1.5 KB of stack writes per step per audio block call — significant overhead at audio rates
- Requires a `MAX_PORTS` constant baked into the hot loop
- `Option<AudioBuffer>` is not `Copy` unless `AudioBuffer` is `Copy` (requires `ptr: *const Sample` instead of `NonNull` for `Copy` derivation, losing niche optimization for the Option)

---

## Approach D: Global Channel Count on the Graph

### Core idea

A single `channels: usize` field on the `Graph` struct. All connections and buffers use this channel count. Buffer allocation becomes `block_size * channels`. The `process()` signature receives a `channels: usize` parameter instead of changing the buffer type.

### process() signature

```rust
fn process(
    &mut self,
    inputs: &[Option<&[Sample]>],   // flat: block_size * channels interleaved or planar
    outputs: &mut [Option<&mut [Sample]>],
    block_size: BlockSize,
    channels: usize,                // NEW
) -> Result<(), AudioNodeRunError>;
```

### Pros / Cons

**Pros**
- Smallest possible change to existing code
- No new types; no new unsafe patterns
- Backwards-compatible for mono (channels=1 is the current behavior)

**Cons**
- Inflexible: cannot mix mono control signals with stereo audio signals in the same graph, a very common DSP requirement (e.g., gain envelope on one channel, audio on another)
- Nodes must document their layout convention (interleaved vs. planar); no type enforcement
- Passing `channels` through the process signature is a leaky abstraction

---

## Approach E: Interleaved Flat Buffer (alternative layout to A)

Same structure as Approach A, but channel samples are interleaved: `[L0, R0, L1, R1, ...]` (LRLR layout) rather than planar.

**Pros over A**
- Familiar to audio hardware / ALSA / PortAudio practitioners

**Cons over A**
- Accessing a single channel requires strided iteration: `data.iter().step_by(channels).skip(c)`
- Worse cache behavior for single-channel DSP operations (must skip over other channel data)
- Channel `c` slice is not contiguous, so `AudioBuffer::channel(c)` cannot return `&[Sample]` without a copy
- The existing channel-slice API (`fn channel(&self, c: usize) -> &[Sample]`) doesn't compose naturally

Planar is the better default for a graph DSP system where nodes typically operate per-channel.

---

## Recommendation

**Approach A** is recommended.

It preserves the zero-cost transmute property that is central to the existing hot-path design. The only new unsafe contract is the `repr(C)` layout of `RawAudioBuffer`/`AudioBuffer`/`AudioBufferMut`, which is a one-time discipline with a straightforward layout test to guard against regressions. The single flat allocation per connection maintains cache locality. The API exposed to node authors (`AudioBuffer::channel(c)`) is clean and obvious.

The required changes are:
1. Add `channels: usize` to `Connection` and port descriptor types
2. Add `RawAudioBuffer`, `AudioBuffer`, `AudioBufferMut` types (with `repr(C)` + layout tests)
3. Change `CompiledStep::input_ptrs` / `output_ptrs` element type from `Option<NonNull<[Sample]>>` to `Option<RawAudioBuffer>`
4. Change buffer allocation in `allocate_empty_buffer_for_connection` to `block_size * channels`
5. Update `compile()` to populate `channels` in each `RawAudioBuffer`
6. Update external slot patching in `run()` to construct `RawAudioBuffer` from caller-supplied `AudioBuffer`/`AudioBufferMut`
7. Update the `AudioNode::process` signature
8. Update all existing node implementations
