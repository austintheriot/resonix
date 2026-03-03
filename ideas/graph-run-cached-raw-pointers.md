# Pre-Cached Raw Pointers for `Graph::run`

## Problem

`Graph::run` currently builds five stack-allocated arrays of size `MAX_PORT_ID = 256` per node per block:

```
output_guards:          [Option<RefMut<'_, [Sample]>>; 256]   // ~6–8 KB
output_buffer_raw_ptrs: [Option<*mut [Sample]>; 256]          // ~4 KB
input_buffer_guards:    [Option<Ref<'_, [Sample]>>; 256]      // ~6–8 KB
input_buffers:          [Option<&[Sample]>; 256]              // ~4 KB
output_buffers:         [Option<&mut [Sample]>; 256]          // ~4 KB
```

This is ~26–28 KB of stack work per node per call, even when a node has only 2 actual ports. A graph with 50 nodes runs ~64,000 closure invocations per block just for array initialization.

## Key Insight

The `Box<[Sample]>` inside each `RefCell<Box<[Sample]>>` in `BufferPool` **never moves between `run` calls**. Reallocation only happens in `allocate_empty_buffer_for_connection`, which is called from `connect()`. The raw pointer to the buffer data is stable across all `run` calls until the graph topology changes.

This means raw pointers can be extracted **once at `connect()` time**, stored in a per-node cache, and used directly during `run` — bypassing `RefCell` borrow acquisition entirely in the hot path.

## Proposed Design

### Cache Structure

Add a per-node cache alongside `NodeConnectionIdMap`, sized to the node's actual port count:

```rust
struct NodeRenderCache {
    // Sized to actual port count at add() time, not MAX_PORT_ID
    input_ptrs:  Box<[Option<*const [Sample]>]>,
    output_ptrs: Box<[Option<*mut   [Sample]>]>,
}
```

Add to `Graph`:

```rust
node_render_cache: IntMap<NodeId, NodeRenderCache>,
```

### Building the Cache

At `connect()` time, after inserting the `ConnectionId` into the port maps, extract and store the raw pointer:

```rust
let raw_ptr: *mut [Sample] = {
    let mut guard = buffer_pool.get(&connection_id).unwrap().borrow_mut();
    &mut **guard as *mut [Sample]
    // guard dropped here — pointer into the Box remains valid
};

node_render_cache[start_node_id].output_ptrs[output_port_id] = Some(raw_ptr);
node_render_cache[end_node_id].input_ptrs[input_port_id] = Some(raw_ptr as *const [Sample]);
```

Fan-out (one output connected to multiple inputs) is naturally handled: multiple downstream nodes store the same `*const [Sample]` in their input cache slots.

### Cache Invalidation

- **`disconnect()` / node removal**: null out the affected cache slots (`None`).
- **`connect()` fan-out**: downstream nodes that previously pointed to an old buffer need their cache updated if the buffer is replaced.

### Self-Loop Handling

Self-loops are detected explicitly at `connect()` time (same node on both sides). The input cache slot for the looped port is set to `None`, preserving the same graceful behavior that `try_borrow` currently provides — decided once rather than on every block.

### External Ports

External output ports receive a caller-supplied buffer per-block and cannot be cached. These continue to be passed via the `outputs: &mut HashMap<ConnectionId, &mut [Sample]>` argument, same as today. The cache only covers internal (pool-backed) connections.

### `run` Loop

The per-node body reduces to:

```rust
let cache = self.node_render_cache.get(&NodeId::from(id)).unwrap();

// Construct &[Sample] / &mut [Sample] from cached raw ptrs
// Wire in external output buffers from the caller for external ports

let ctx = AudioNodeContext {
    input_buffers:  /* from cache.input_ptrs */,
    output_buffers: /* from cache.output_ptrs + external overrides */,
    block_size: self.block_size,
};
node.process(ctx)?;
```

No `core::array::from_fn`, no `RefCell` borrow/release, no guard arrays, no per-port HashMap probes.

## Safety Contract

The `unsafe` is justified by three invariants:

1. **Buffers don't move during `run`** — `allocate_empty_buffer_for_connection` is only called from `connect()`, which is never called concurrently with or during `run`. The `Box<[Sample]>` heap allocation is stable for the lifetime of the connection.

2. **No mutable aliasing on outputs** — no two output port slots share a `ConnectionId` (existing invariant). Each `*mut [Sample]` in `output_ptrs` is unique across all nodes.

3. **Visit order respects data flow** — nodes are processed in topological order. By the time a downstream node reads a buffer as input, the upstream node has already finished writing to it. No two nodes simultaneously access the same buffer with conflicting access.

## Tradeoffs

**Gained:**
- O(actual port count) work per node per block instead of O(MAX_PORT_ID)
- No `RefCell` borrow/release in the hot path
- No large stack arrays
- No per-port HashMap probes for buffer lookup

**Lost:**
- Safety is now structural (invariants upheld by `connect()`/`disconnect()` logic) rather than enforced dynamically by `RefCell`. A stale pointer from a missed cache invalidation would be UB rather than a panic.
- More cache maintenance logic in `connect()` and `disconnect()`.
- The `RefCell` served as a signal that "this memory is shared." With raw pointers, that intent moves entirely to comments and cache maintenance.

## Relation to `AudioNodeContext` API

This change is compatible with either the current fixed-size `[Option<...>; MAX_PORT_ID]` API or a future slice-based API. Combining with a slice-based `AudioNodeContext` (passing `&[Option<&[Sample]>]` instead of fixed arrays) would eliminate the last remaining O(MAX_PORT_ID) initialization — converting raw pointers to references for `process()`.
