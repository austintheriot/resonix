# Compiled Linear Execution Plan for `Graph::run`

## Problem

`Graph::run` does significant bookkeeping work per node per audio block, none of which changes
between blocks when the graph topology is stable:

- Two `IntMap` lookups per node (one into `graph_items`, one into `node_connection_id_map`)
- Five `core::array::from_fn` initializations over `MAX_PORT_ID = 256` slots each
- Multiple `RefCell::try_borrow` / `try_borrow_mut` calls per connected port

Most of this work is pure repetition of decisions already settled at `connect()` time. The graph
topology doesn't change between `run()` calls. The buffer addresses don't move. The visit order
is fixed. The only thing that legitimately changes per block is external output buffers supplied
by the caller.

## Core Idea

Compile the graph into a flat `Vec<CompiledStep>` after each topology change, encoding every
per-node decision (node pointer, input buffer pointers, output buffer pointers) into pre-baked
structs. `run()` then becomes a linear scan with essentially no decision-making:

```rust
for step in &mut self.compiled_steps {
    // patch external outputs from caller (O(external_port_count) per node)
    for &(slot, conn_id) in step.external_output_slots.iter() {
        step.output_ptrs[slot] = outputs.get_mut(&conn_id).map(NonNull::from);
    }

    // transmute stored NonNull ptrs to typed references (see below)
    let inputs:  &    [Option<&    [Sample]>] = unsafe { transmute(step.input_ptrs.as_ref())  };
    let outputs: &mut [Option<&mut [Sample]>] = unsafe { transmute(step.output_ptrs.as_mut()) };

    unsafe { (&mut *step.node).process(inputs, outputs, step.block_size)?; }
}
```

## Required Structures

### `CompiledStep`

```rust
struct CompiledStep {
    // Fat pointer (data_ptr, vtable_ptr) into the Box<dyn AudioNode>'s heap allocation.
    // Stable: moving a Box in the IntMap does not move the heap allocation it points to.
    node: *mut dyn AudioNode,

    // Sized to actual input port count (not MAX_PORT_ID).
    // None = port exists but is unconnected (or self-loop).
    input_ptrs: Box<[Option<NonNull<[Sample]>>]>,

    // Sized to actual output port count (not MAX_PORT_ID).
    // Slots for external ports are left as None here; patched per-run from caller.
    output_ptrs: Box<[Option<NonNull<[Sample]>>]>,

    // Which slots in output_ptrs are external, and their ConnectionId for lookup.
    external_output_slots: Box<[(usize, ConnectionId)]>,

    block_size: BlockSize,
}
```

### `Graph` addition

```rust
compiled_plan: Option<Vec<CompiledStep>>,  // None = needs recompilation
```

Invalidated (set to `None`) by `add()`, `connect()`, and any future `disconnect()`/`remove()` —
the same triggers that currently invalidate `visit_order`. Recompiled lazily on the next `run()`
call, or eagerly if preferred.

## Why `NonNull` Instead of Raw Pointers

`Option<*const [Sample]>` and `Option<&[Sample]>` have **different sizes**:

- `&[Sample]` is a fat pointer (data + length = 2 × usize = 16 bytes on 64-bit).
  `Option<&[Sample]>` uses the null-pointer niche — both the data and length fields
  being zero represents `None` — so `size_of::<Option<&[Sample]>>() == 16`.
- `*const [Sample]` is also a fat pointer, but raw pointers carry no non-null guarantee.
  The compiler cannot use the niche, so it must add a discriminant byte and pad to alignment.
  `size_of::<Option<*const [Sample]>>() == 24`.

Since `Option<*const [Sample]>` and `Option<&[Sample]>` have different sizes, a slice of one
cannot be safely transmuted to a slice of the other.

`NonNull<[Sample]>` is a non-null fat pointer — it participates in the same null-pointer niche
optimization as references:

```
size_of::<Option<NonNull<[Sample]>>>() == 16   // same as Option<&[Sample]>
```

`Option<NonNull<[Sample]>>`, `Option<&[Sample]>`, and `Option<&mut [Sample]>` are all 16 bytes
with identical bit-layout. The transmute is a zero-cost type annotation: no instructions emitted,
no data moved.

## Why Transmute at All

Given that the layouts match, why not just dereference?

Dereferencing `NonNull<[Sample]>` yields `[Sample]` (an unsized type), which requires a borrow:
`unsafe { &*ptr.as_ptr() }`. That borrow has a lifetime attached to the call site, not to the
`CompiledStep`. To turn a `Box<[Option<NonNull<[Sample]>>]>` into a `&[Option<&[Sample]>]` by
dereferencing each element, you would need to iterate the slice, materialize each reference, and
collect into a new allocation — defeating the entire purpose of pre-baking the data.

Transmute reinterprets the already-existing memory of `input_ptrs` as the correct reference type
in place. No allocation. No iteration. The `Box<[Option<NonNull<[Sample]>>]>` stored in the
`CompiledStep` is the same memory the `process` call reads from; we're just telling the type
system what it already is at the bit level.

## Safety Contract

Three invariants must hold for the transmute and node pointer dereference to be sound:

1. **Buffer addresses are stable for the duration of `run()`.**
   `allocate_empty_buffer_for_connection` is only ever called from `connect()`.
   No topology modification (add/connect/disconnect/remove) happens concurrently with `run()`.
   Therefore the heap allocation behind each `Box<[Sample]>` in `BufferPool` does not move
   during any `run()` call, and every stored `NonNull<[Sample]>` remains valid.

2. **No mutable aliasing on output buffers.**
   No two output port slots in the entire plan share the same `ConnectionId`, and therefore no
   two `NonNull` pointers in `output_ptrs` across all steps alias the same memory. The transmute
   to `&mut [Sample]` is safe because Rust's alias rules are satisfied by the graph invariant
   that each buffer has exactly one producer.

3. **Visit order enforces exclusive access ordering.**
   The compiled steps are ordered topologically (same ordering as `visit_order`). By the time a
   step reads a buffer as input, the upstream step that writes it as output has already
   completed. No two steps simultaneously hold conflicting access to the same buffer.

## `process` API Requirement

This proposal reaches its full potential only when paired with a slice-based `process` signature:

```rust
fn process(
    &mut self,
    inputs:     &[Option<&[Sample]>],
    outputs: &mut[Option<&mut [Sample]>],
    block_size: BlockSize,
) -> Result<(), AudioNodeRunError>;
```

With the current `AudioNodeContext` holding `[Option<...>; MAX_PORT_ID]` fixed-size arrays,
`run()` would still need to build 256-element arrays per node per call (from the pre-cached
pointers, which is faster than the current RefCell path, but still O(256)). The slice-based
signature lets each node receive a slice exactly as long as its port count, and the transmute
from `CompiledStep` storage produces that slice directly with no intermediate allocation.

## Dense Per-Direction Port IDs

Port IDs must be dense within each direction namespace (inputs: `0..n`, outputs: `0..m`) for
the slice lengths to equal the actual port counts. The current codebase uses a flat integer
namespace shared across all port directions, so some nodes have gaps (e.g., `ConstantNode`'s
only output sits at port ID 1, leaving slot 0 unused in the output array).

This is enforced at `add()` time: after calling `describe_ports()`, validate that input port IDs
form an unbroken sequence starting at 0, and likewise for output port IDs.

`count_ports` already computes `max_port_id + 1` per direction group. With dense IDs, that
value equals the actual port count. Without dense IDs, it overallocates.

One site requires fixing before this enforcement is safe: `NodeHandle::external_connection_ids`
is currently an `IntMap<PortId, ConnectionId>` that conflates external inputs and external
outputs under the same key type. With per-direction dense IDs, a node with both an external
input and an external output would have both at port ID 0, colliding in this map. Fix by
splitting into `external_input_connection_ids` and `external_output_connection_ids`.

## Per-Run Work After Compilation

For each node:
- **O(external_output_count):** patch external output slots from caller's `outputs` map
- **O(1):** two `transmute` calls (zero instructions)
- **O(1):** one vtable dispatch into `process`

Versus current per-node work:
- **O(1):** two `IntMap` lookups (hash + probe)
- **O(MAX_PORT_ID) = O(256):** five `from_fn` initializations
- **O(connected_port_count):** `RefCell::try_borrow` / `try_borrow_mut` per port

## Tradeoffs

**Gained:**
- Eliminates per-node map lookups in the hot path entirely
- Eliminates all fixed-size 256-element array initialization per node per block
- Eliminates `RefCell` borrow/release in the hot path
- Better cache locality: compiled steps are sequential in memory; current approach
  pointer-chases through `IntMap` on every node

**Lost:**
- The `RefCell` served as a dynamic aliasing check. Its removal moves correctness entirely
  into structural invariants enforced at `connect()` time and verified through `unsafe` reasoning.
  A stale pointer from a missed cache invalidation is undefined behavior rather than a panic.
- `compiled_plan` must be kept in sync with topology. Every path that currently invalidates
  `visit_order` must also invalidate `compiled_plan`. Missing one is a latent UB bug.
- `*mut dyn AudioNode` loses Rust's borrow-checker protection for the node's own data. The
  safety argument depends on the single-threaded, non-reentrant `run()` contract.
- The `process` signature change and dense port ID enforcement are prerequisite work that
  touches every node implementation and breaks callers using the old API.
