# NodeProcessContext: Hiding Guards & Eliminating RefCell

Follow-up to `node-process-context.md`.

---

## Hiding `Ref`/`RefMut` from `AudioNode`

`NodeProcessContext` can own the guards internally and expose clean slices:

```rust
pub struct NodeProcessContext<'pool> {
    inputs: [Option<Ref<'pool, AudioBuffer>>; MAX_PORTS],
    outputs: [Option<RefMut<'pool, AudioBuffer>>; MAX_PORTS],
    block_size: BlockSize,
}

impl<'pool> NodeProcessContext<'pool> {
    pub fn input(&self, port: PortId) -> Option<&[f32]> { ... }
    pub fn output(&mut self, port: PortId) -> Option<&mut [f32]> { ... }
}
```

Node implementations never see `Ref`/`RefMut`. But there's a subtle ergonomics problem:
you can't hold an input and an output reference simultaneously through the context,
because `input` borrows `&self` and `output` borrows `&mut self`. A multi-port node doing:

```rust
let input = ctx.input(PORT_0)?;   // borrows ctx
let output = ctx.output(PORT_0)?; // can't: needs &mut ctx
```

...won't compile. You'd have to read then write, never overlapping. For DSP that's often
fine but it's a real constraint worth knowing upfront.

---

## Assumption to challenge: `NodeProcessContext` may not be needed at all

The existing `AudioNode::process` signature:

```rust
fn process(&mut self, inputs: &[&[Sample]], outputs: &mut [&mut [Sample]]) -> Result<(), AudioNodeRunError>;
```

...already gives nodes clean, direct slice access — no guards, no context struct. The node
implementation is as simple as it gets.

The complexity of collecting buffer references is a `Graph::run` problem. It doesn't have
to leak into the `AudioNode` abstraction. If `Graph::run` builds the slices correctly before
calling `process`, nodes stay clean and the bookkeeping lives where it belongs.
`NodeProcessContext` is an extra layer that may not be earning its keep.

---

## Assumption to challenge: `RefCell` is required for safety

It isn't. `RefCell` is there to satisfy the borrow checker — not because there's a real
aliasing hazard.

The actual invariant: when node B is processing, its input buffers were written by
predecessor nodes (already complete), and its output buffers are written only by node B.
No two nodes touch the same buffer simultaneously. This is guaranteed by the topological
visit order you're already computing.

`RefCell` is runtime enforcement of something statically true by graph construction.

---

## Paths that eliminate `RefCell`

### Option A: `unsafe` in one clearly-bounded place

A node's input and output buffer indices are always disjoint — provably true from graph
structure (a connection goes from one node's output to another node's input; same buffer
can't be both input and output of the same node, barring cycles which are already
rejected). That invariant can be expressed in one `unsafe` block in `Graph::run` and
nowhere else:

```rust
// Invariant: output_idx and all input_idxs are distinct — proven by graph structure
let output_buf: &mut [f32] = unsafe { &mut *buffer_pool.get_mut_ptr(output_idx) };
let input_buf: &[f32] = unsafe { &*buffer_pool.get_ptr(input_idx) };
```

The unsafe is auditable and localized. The rest of the codebase stays safe.

### Option B: `slice::get_many_mut` on a Vec-backed pool

If `BufferPool` is a `Vec<AudioBuffer>` instead of a map of `RefCell`s, and you know the
distinct indices of all buffers a node needs, `get_many_mut` (stabilized in Rust 1.86) gets
non-overlapping mutable references to distinct elements simultaneously — no unsafe, no
`RefCell`. Would require changing `BufferPool`'s backing store and using a
`ConnectionId -> usize` index map.

### Option C: Pre-copy inputs into scratch

Before calling `process`, copy each input buffer into a per-node scratch area. The node
reads from scratch (immutably borrowed) and writes to its output buffers (mutably
borrowed). No aliasing, no `RefCell`, no unsafe. Cost is a `memcpy` per buffer per block —
acceptable depending on block size and graph density.

---

## The harder structural problem

None of the above fully resolves the two-pass borrow issue in `run`: you need port
addresses from the node to look up buffers, then need `&mut node` to call `process`. These
two needs conflict on `&mut self`.

The cleanest long-term fix: store port address information redundantly outside the node.
`port_address_to_connection_id_map` already does this partially — extend that pattern so
`run` never needs to call into the node to discover its ports, only to call `process`.
