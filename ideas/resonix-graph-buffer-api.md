# Graph Buffer API: Zero-Allocation Design Proposals

*Date: 2026-02-26*

---

## Problem Statement

`Graph::run()` currently allocates on every call. The hotspots, per node per frame:

```rust
// Allocation 1 & 2: slice-of-slices constructed fresh each frame
let inputs_refs: Vec<&[Sample]> =
    self.run_inputs.iter().map(|v| v.as_slice()).collect();
let mut outputs_refs: Vec<&mut [Sample]> =
    self.run_outputs.iter_mut().map(|v| v.as_mut_slice()).collect();

// Allocation 3 & 4: output buffer content cloned into connection map and external outputs
self.run_connections_sample_map.insert(*connection_id, output_block.clone());
outputs.insert(*external_output_port_address, output_block.clone());
```

Goal: **zero heap operations inside the run loop**. Graph structure changes (add/connect) may allocate; `run()` must not.

---

## Two Orthogonal Design Decisions

The full solution requires choosing one answer to each:

### Decision 1: Buffer Ownership Model

Who owns the audio data buffers, and how is fan-out (one output → many inputs) handled?

| Model | Who owns buffers | Fan-out cost |
|---|---|---|
| **A. Per-connection** (current shape) | Each wire has its own `Vec<Sample>` | N clones |
| **B. Per-output-port** (Pd-style) | Each output *port* has one `Vec<Sample>`; connections are just references to it | Zero — all readers share the same memory |
| **C. Flat slab** | One contiguous allocation: `N_buffers × block_size` | Zero with model B semantics; better cache locality |

Model B eliminates allocations 3 & 4 entirely. Model C is an optimization on top of B.

### Decision 2: Trait API Shape

How does `AudioNode::process` receive its buffers?

| Shape | How node gets buffers | How run loop builds args |
|---|---|---|
| **X. Slice-of-slices (current)** | `inputs: &[&[Sample]]` | Must build a `Vec<&[Sample]>` each frame — the allocation |
| **Y. ProcessContext** | `ctx: &mut ProcessContext<'_>` with `ctx.inputs`, `ctx.outputs`, `ctx.block_size` | Struct assembled from pre-allocated per-node data — zero allocation |
| **Z. SmallVec** | Same as X | `SmallVec<[&[Sample]; 8]>` — stack-allocated for ≤8 ports |

---

## Concrete Proposals

### Proposal 1: ProcessContext + Per-Output-Port Flat Pool
*"The Rustified Pd model"*

**Buffer model B+C, Trait shape Y.**

Pre-allocate one contiguous slab at graph-build time. Each output port maps to a
fixed offset in the slab. Connections are recorded as `(src_offset, dst_port_index)`
mappings — no buffers, just bookkeeping.

At graph-build time, for each node in visit order, pre-compute:
- Which slab offsets are its inputs (the output-port offsets of upstream nodes)
- Which slab offset is its output (its own output port's offset)

These are stored in a compiled `ExecPlan`. The run loop iterates the plan, assembles a
`ProcessContext` struct (stack-allocated, no heap), and calls `process`.

```rust
// Pre-allocated once at graph-build time
pub struct ExecPlan {
    /// Flat contiguous sample pool: all signal buffers back-to-back
    pool: Vec<Sample>,       // len = num_output_ports * block_size
    block_size: usize,
    entries: Vec<ExecEntry>, // in topological order
}

pub struct ExecEntry {
    node_id: NodeId,
    /// Offsets into `pool` for each input port.
    /// Multiple entries may share the same offset (fan-in from same source = same buffer).
    input_offsets: SmallVec<[usize; 4]>,
    /// Offsets into `pool` for each output port (unique per output port).
    output_offsets: SmallVec<[usize; 4]>,
}

// Assembled on the stack during run() — no heap allocation
pub struct ProcessContext<'a> {
    pub inputs: &'a [&'a [Sample]],
    pub outputs: &'a mut [&'a mut [Sample]],
    pub block_size: usize,
}
```

**The run loop:**
```rust
fn run(&mut self) -> Result<(), GraphRunError> {
    let plan = &mut self.exec_plan;
    let block_size = plan.block_size;

    for entry in &plan.entries {
        let node = get_node_mut(&mut self.nodes, entry.node_id);

        // Build slice-of-slice arrays from pre-computed offsets.
        // SmallVec means no heap allocation for ≤4 ports.
        // This is O(n_ports) pointer arithmetic, not allocation.
        let mut input_slices: SmallVec<[&[Sample]; 4]> = entry
            .input_offsets
            .iter()
            .map(|&off| &plan.pool[off..off + block_size])
            .collect();

        let mut output_slices: SmallVec<[&mut [Sample]; 4]> = entry
            .output_offsets
            .iter()
            .map(|&off| {
                // SAFETY: output offsets are unique per node — no two nodes
                // share an output offset. Input offsets point to upstream
                // output regions which this node never writes. No aliasing.
                unsafe {
                    let ptr = plan.pool.as_mut_ptr().add(off);
                    core::slice::from_raw_parts_mut(ptr, block_size)
                }
            })
            .collect();

        let ctx = ProcessContext {
            inputs: &input_slices,
            outputs: &mut output_slices,
            block_size,
        };
        node.process(ctx)?;
    }
    Ok(())
}
```

**The trait:**
```rust
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(&mut self, ctx: &mut ProcessContext<'_>) -> Result<(), AudioNodeRunError>;
}
```

**Example node** (note: destructure ctx to avoid borrow conflicts):
```rust
impl AudioNode for MultiplyNode {
    fn process(&mut self, ctx: &mut ProcessContext<'_>) -> Result<(), AudioNodeRunError> {
        let ProcessContext { inputs, outputs, block_size } = ctx;
        let left  = inputs.get(0).map(|s| *s).unwrap_or(&[]);
        let right = inputs.get(1).map(|s| *s).unwrap_or(&[]);
        let out   = &mut outputs[0];

        for i in 0..*block_size {
            out[i] = Sample::new(
                *left.get(i).copied().unwrap_or(self.left_default)
                    * *right.get(i).copied().unwrap_or(self.right_default),
            );
        }
        Ok(())
    }
}
```

**Key properties:**
- Fan-out is zero-copy: multiple `input_offsets` entries in downstream nodes point to the same pool region
- The unsafe in the run loop is localized and auditable (not in node code)
- SmallVec eliminates heap allocation for most nodes (≤4 ports)
- The flat pool gives good cache behavior: signal data is contiguous

**Downsides:**
- Breaking change to `AudioNode` trait
- Unsafe in the graph run loop (aliasing argument must be maintained by graph invariants)
- `ProcessContext` destructuring is slightly awkward compared to plain slices
- SmallVec still allocates on the heap for nodes with >4 ports

---

### Proposal 2: Pre-Compiled Pointer Arrays, Keep Current Trait
*"Pd-faithful, minimal API surface change"*

**Buffer model B+C, Trait shape X.**

Keep `&[&[Sample]]` / `&mut [&mut [Sample]]` in the trait — nodes written today still compile.
Eliminate the per-frame `Vec<&[Sample]>` allocation by pre-computing per-node pointer/length arrays
at graph-build time and updating them (just pointer writes, no allocation) at the start of each
frame.

```rust
pub struct ExecPlan {
    pool: Vec<Sample>,          // flat: num_output_ports * block_size
    block_size: usize,
    entries: Vec<ExecEntry>,
}

pub struct ExecEntry {
    node_id: NodeId,
    /// Pre-allocated Vecs that are REUSED across frames.
    /// Contents are raw fat-pointer parts; updated each frame before calling process.
    ///
    /// Stored as (*const Sample, usize) pairs so we can update the pointer
    /// without reallocating the Vec.
    input_fat_ptrs: Vec<(*const Sample, usize)>,
    output_fat_ptrs: Vec<(*mut Sample, usize)>,
    input_offsets: SmallVec<[usize; 4]>,
    output_offsets: SmallVec<[usize; 4]>,
}
```

**The run loop:**
```rust
fn run(&mut self) -> Result<(), GraphRunError> {
    let pool_ptr = self.exec_plan.pool.as_mut_ptr();
    let block_size = self.exec_plan.block_size;

    for entry in &mut self.exec_plan.entries {
        // Update pre-allocated fat-pointer arrays (O(ports) pointer writes, zero allocation)
        for (i, &off) in entry.input_offsets.iter().enumerate() {
            entry.input_fat_ptrs[i] = (unsafe { pool_ptr.add(off) }, block_size);
        }
        for (i, &off) in entry.output_offsets.iter().enumerate() {
            entry.output_fat_ptrs[i] = (unsafe { pool_ptr.add(off) as *mut Sample }, block_size);
        }

        // Transmute pre-allocated Vec<(*const Sample, usize)> → &[&[Sample]]
        // A fat pointer IS a (*const T, usize) pair in Rust's layout.
        // SAFETY: same aliasing argument as Proposal 1; pool offsets are disjoint for outputs.
        let inputs: &[&[Sample]] = unsafe {
            core::slice::from_raw_parts(
                entry.input_fat_ptrs.as_ptr() as *const &[Sample],
                entry.input_fat_ptrs.len(),
            )
        };
        let outputs: &mut [&mut [Sample]] = unsafe {
            core::slice::from_raw_parts_mut(
                entry.output_fat_ptrs.as_mut_ptr() as *mut &mut [Sample],
                entry.output_fat_ptrs.len(),
            )
        };

        let node = get_node_mut(&mut self.nodes, entry.node_id);
        node.process(inputs, outputs)?;
    }
    Ok(())
}
```

**Key properties:**
- **No change to `AudioNode` trait** — existing nodes compile unchanged
- Zero allocation in run loop: the `Vec<(*const Sample, usize)>` per entry is pre-allocated and reused
- The transmute is technically unsound without a layout guarantee. Rust does not currently guarantee that `&[T]` has the same layout as `(*const T, usize)`, although this is true in practice on all targets
- Could use `core::mem::transmute_copy` or pointer casting as a more careful variant

**Downsides:**
- The transmute is load-bearing and would be UB if the fat-pointer layout assumption ever changes
- Reasoning about which unsafe code is "safe" is harder
- Still requires the `input_fat_ptrs` / `output_fat_ptrs` Vecs to be pre-allocated (done at graph build time), so `add()`/`connect()` must maintain them

---

### Proposal 3: SmallVec + Pre-Allocated Connection Pool
*"Minimal change, high confidence, nearly zero allocation"*

**Buffer model A (per-connection), Trait shape Z (SmallVec).**

The least invasive option. Two targeted fixes:

**Fix 1**: Replace the per-frame `Vec<&[Sample]>` construction with `SmallVec`.

```rust
// Before (allocates):
let inputs_refs: Vec<&[Sample]> = self.run_inputs.iter().map(|v| v.as_slice()).collect();

// After (stack-allocated for ≤8 ports):
let inputs_refs: SmallVec<[&[Sample]; 8]> = self.run_inputs.iter().map(|v| v.as_slice()).collect();
let mut outputs_refs: SmallVec<[&mut [Sample]; 8]> = self.run_outputs.iter_mut().map(|v| v.as_mut_slice()).collect();
```

**Fix 2**: Pre-allocate connection buffers at `connect()` time. Replace the cleared-and-reinserted `run_connections_sample_map` with a stable map populated once:

```rust
// In Graph struct:
// Before: IntMap<ConnectionId, Vec<Sample>> — cleared and re-inserted each frame
// After:  IntMap<ConnectionId, Vec<Sample>> — populated once at connect(), never reallocated

// At connect():
self.connection_buffers.insert(new_connection_id, vec![Sample::default(); self.block_size]);

// In run() — no clone, no insert:
if let Some(buf) = self.connection_buffers.get_mut(&connection_id) {
    buf.copy_from_slice(&self.run_outputs[port_id]); // memcpy, no allocation
}
```

**Also add `block_size` to the trait:**

```rust
pub trait AudioNode: ... {
    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError>;
}
```

The `block_size` parameter lets nodes avoid iterating the full buffer capacity on short frames.
It's a minor API change but one we'd need eventually regardless.

**Key properties:**
- External outputs (`HashMap<PortAddress, Vec<Sample>>`) still clone — but this is at the graph
  boundary, not the hot path, and could be fixed separately
- SmallVec still allocates for nodes with >8 ports (unusual in practice)
- Connection buffers still require a `copy_from_slice` per connection per frame (not zero-copy)
  but at least it's no longer a heap operation
- Fastest path to "mostly allocation-free" with lowest risk

**Downsides:**
- Not truly zero-allocation (SmallVec spills, external output clones)
- Not zero-copy (still `copy_from_slice` per connection)
- Doesn't solve fan-out efficiently (N connections from one output = N copies)
- `block_size` parameter adds noise to every node implementation

---

### Proposal 4: Static Dispatch Perform Chain
*"Maximum throughput, closest to Pd internals, major architectural shift"*

**Buffer model C (flat slab), no trait dispatch.**

At graph-build time, compile the graph into a flat `Vec<PerformOp>`. Each op holds a type-erased function pointer + raw pointers to its input/output regions in the pool. No dynamic dispatch during `run()` — just a linear array of function calls.

```rust
type PerformFn = unsafe fn(
    state: *mut (),
    inputs: *const *const Sample,
    outputs: *const *mut Sample,
    n_inputs: usize,
    n_outputs: usize,
    block_size: usize,
);

pub struct PerformOp {
    func: PerformFn,
    state: *mut (),              // erased pointer to the node's state
    inputs: Box<[*const Sample]>,  // pre-computed, pointing into pool
    outputs: Box<[*mut Sample]>,   // pre-computed, pointing into pool
    block_size: usize,
}

pub struct PerformChain {
    pool: Vec<Sample>,
    ops: Vec<PerformOp>,
}
```

**A node "registers" itself at graph-compile time:**
```rust
// Generated via macro or manual impl:
unsafe fn multiply_perform(
    state: *mut (),
    inputs: *const *const Sample,
    outputs: *const *mut Sample,
    n_in: usize, n_out: usize, block_size: usize,
) {
    let node = &mut *(state as *mut MultiplyNode);
    let in0 = core::slice::from_raw_parts(*inputs.add(0), block_size);
    let in1 = core::slice::from_raw_parts(*inputs.add(1), block_size);
    let out = core::slice::from_raw_parts_mut(*outputs.add(0), block_size);
    for i in 0..block_size {
        out[i] = Sample::new(*in0[i] * *in1[i]);
    }
}
```

**Run loop:**
```rust
fn run(&mut self) -> Result<(), GraphRunError> {
    for op in &self.chain.ops {
        unsafe {
            (op.func)(
                op.state,
                op.inputs.as_ptr(),
                op.outputs.as_ptr(),
                op.inputs.len(),
                op.outputs.len(),
                op.block_size,
            );
        }
    }
    Ok(())
}
```

**Key properties:**
- No dynamic dispatch (no vtable lookups per node)
- Zero allocation in run loop
- Cache-friendly: linear iteration through `ops`
- Exactly the Pd `dsp_tick()` model in Rust

**Downsides:**
- Requires all unsafe to be in node code — the trait-based safety model is gone
- `*mut ()` for state is unsound without careful lifetime management
- Rebuilding the chain on every `add()`/`connect()` is expensive (same as Pd's "recompile DSP")
- Live graph modification becomes significantly harder
- Debugging is painful — crash locations point into function pointers
- Essentially abandons the ergonomic Rust trait system in favour of C-style pointers

---

## Comparison

| | Proposal 1 (ProcessContext) | Proposal 2 (Ptr Arrays) | Proposal 3 (SmallVec) | Proposal 4 (Perform Chain) |
|---|---|---|---|---|
| **Allocations in run loop** | Zero (SmallVec ≤4) | Zero | Near-zero (SmallVec ≤8) | Zero |
| **Fan-out cost** | Zero copy | Zero copy | N `copy_from_slice` | Zero copy |
| **Trait changes** | Breaking | None | Minor (`block_size`) | Eliminated |
| **Unsafe** | In graph internals | In graph internals (stronger) | None | In node code |
| **Node ergonomics** | Good (destructure ctx) | Unchanged | Unchanged | Poor (raw ptrs) |
| **Implementation risk** | Medium | High (layout assumption) | Low | High |
| **Cache behavior** | Flat pool — excellent | Flat pool — excellent | Per-connection — ok | Flat pool — excellent |

---

## Recommendation: Start with Proposal 1, optionally adopt Proposal 2 later

**Proposal 1 (ProcessContext + flat pool)** is the target architecture. The reasons:

1. **Zero allocation, zero copy fan-out** — hits the primary goal
2. **Unsafe is contained** in the graph run loop, not node implementations
3. **ProcessContext is extensible** — adding `sample_rate`, `tempo`, or control-rate inputs later is just adding a field, not changing the whole trait
4. **Flat pool** is cache-friendly and sets up SIMD-aligned allocation later (align the pool to 32 bytes)
5. The layout transmute in Proposal 2 is a bet against future Rust ABI changes; worth avoiding

**Proposal 3 (SmallVec)** is a reasonable stepping stone — it can be done incrementally today
with low risk, and sets up the pre-allocated connection buffer model that Proposal 1 also needs.

**Proposal 4 (Perform Chain)** should be considered only if profiling reveals that vtable
dispatch is a bottleneck in practice (unlikely until graphs have hundreds of nodes).

---

## Open Questions

1. **External output API**: The `HashMap<PortAddress, Vec<Sample>>` that `run()` currently populates for the host still clones. Should this be a pre-allocated output slab instead, with the host given a read-only view after each frame?

2. **Graph mutation during DSP**: With a compiled `ExecPlan`, structural changes (add/connect) must rebuild it. Should this be atomic (swap ExecPlan pointer from a different thread), or is it acceptable to require pausing the run loop?

3. **Control-rate inputs to audio nodes**: The `ParamNode` distinction already exists. Should `ProcessContext` carry a separate `control_inputs: &[Sample]` slice for scalar (non-block) parameters? This would cleanly map to Pd's message vs signal domain split.

4. **SmallVec N**: Both proposals 1 and 3 use SmallVec. What's the right inline size? Histogram across real patches needed — likely 4 is fine for audio nodes, 2 for control nodes.

5. **Cycle handling**: Cyclic nodes currently get zeroed inputs at frame start. With a flat pool, zeroing the cyclic node's input region before the frame is still needed. How does the ExecPlan mark which regions to zero?

---

## References

- Current trait: `crates/resonix-graph/src/traits/audio_node.rs`
- Current run loop: `crates/resonix-graph/src/implementations/graph.rs:387`
- Pure Data signal model: `ideas/pd-analysis/PURE_DATA_KEY_FINDINGS.md` §3, §4
- Prior buffer proposals: `ideas/resonix-pre-allocated-buffers.md`
