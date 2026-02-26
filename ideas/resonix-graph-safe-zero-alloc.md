# Graph Buffer API: Safe Zero-Allocation Design

*Date: 2026-02-26*

---

## Summary

A fully safe (no `unsafe` blocks), zero-allocation-in-run-loop design for `Graph::run()`.

The key techniques:
- Per-node pre-allocated scratch buffers (`input_scratch`, `output_scratch`)
- Topological execution order + `split_at_mut` to prove non-aliasing to the borrow checker
- `SmallVec` to stack-allocate slice-of-slice arguments for ≤4 ports
- Free list for buffer reuse on node deletion

**Trade-off vs unsafe flat slab**: fan-out is N memcopies instead of zero-copy. At typical block sizes (64–512 samples × 4 bytes), this is cheap and cache-friendly — far less expensive than heap allocation.

---

## Data Structures

```rust
pub struct ExecPlan {
    entries: Vec<ExecEntry>,    // in topological order
    block_size: usize,
    free_pool: Vec<Box<[Sample]>>,  // recycled scratch buffers
}

pub struct ExecEntry {
    node_id: NodeId,
    /// Pre-allocated input scratch: n_inputs * block_size samples.
    /// Upstream outputs are gathered into this before calling process().
    input_scratch: Box<[Sample]>,
    /// Pre-allocated output scratch: n_outputs * block_size samples.
    /// process() writes here; downstream nodes gather from it.
    output_scratch: Box<[Sample]>,
    /// Which upstream outputs map to which of my input slots.
    input_connections: SmallVec<[InputConnection; 4]>,
    n_inputs: usize,
    n_outputs: usize,
}

struct InputConnection {
    /// Index into ExecPlan::entries (must be < this entry's index — topological invariant).
    source_entry_idx: usize,
    source_output_slot: usize,
    my_input_slot: usize,
}
```

---

## Run Loop (fully safe)

```rust
fn run(&mut self) -> Result<(), GraphRunError> {
    let block_size = self.exec_plan.block_size;
    let entries = &mut self.exec_plan.entries;

    for i in 0..entries.len() {
        // split_at_mut gives the compiler proof that `done` and `entry` are
        // non-overlapping — no unsafe needed.
        let (done, not_done) = entries.split_at_mut(i);
        let (entry, _) = not_done.split_first_mut().unwrap();

        // Gather: copy each upstream output slot into our input_scratch.
        // `done[j]` is an immutable borrow — safe because done ≠ current.
        for conn in &entry.input_connections {
            let src = &done[conn.source_entry_idx].output_scratch
                [conn.source_output_slot * block_size..][..block_size];
            let dst = &mut entry.input_scratch
                [conn.my_input_slot * block_size..][..block_size];
            dst.copy_from_slice(src);
        }

        // Build input slice-of-slices (SmallVec: stack-allocated for ≤4 ports).
        let mut inputs: SmallVec<[&[Sample]; 4]> = SmallVec::new();
        for slot in 0..entry.n_inputs {
            inputs.push(&entry.input_scratch[slot * block_size..][..block_size]);
        }

        // Build output slice-of-slices via sequential split_at_mut — no unsafe.
        let mut outputs: SmallVec<[&mut [Sample]; 4]> = SmallVec::new();
        let mut scratch = &mut entry.output_scratch[..];
        for _ in 0..entry.n_outputs {
            let (chunk, rest) = scratch.split_at_mut(block_size);
            outputs.push(chunk);
            scratch = rest;
        }

        let node = self.nodes.get_mut(entry.node_id).unwrap();
        node.process(&inputs, &mut outputs)?;
    }
    Ok(())
}
```

### Why this is safe

| Concern | Why it's fine |
|---|---|
| `done[j].output_scratch` read while `entry.input_scratch` is mutated | Different `ExecEntry` instances; `split_at_mut` proves they don't overlap |
| Multiple `&mut [Sample]` from `output_scratch` | Sequential `split_at_mut` chain — the compiler tracks non-overlap |
| `&[Sample]` inputs and `&mut [Sample]` outputs simultaneously | `input_scratch` and `output_scratch` are separate `Box<[Sample]>` allocations |

---

## AudioNode Trait

No change required to the signature — nodes continue to receive `&[&[Sample]]` and `&mut [&mut [Sample]]`:

```rust
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
    ) -> Result<(), AudioNodeRunError>;
}
```

The only optional addition is `block_size` if nodes need it (currently they can infer it from `outputs[0].len()`).

---

## Buffer Lifecycle: Free List

Arena allocation doesn't work well for graphs with node deletion because deleted nodes leave holes. Instead, use a simple free list:

```rust
impl ExecPlan {
    fn acquire_scratch(&mut self, n_slots: usize) -> Box<[Sample]> {
        let needed = n_slots * self.block_size;
        // Reuse a buffer of matching size if available.
        self.free_pool
            .iter()
            .position(|b| b.len() == needed)
            .map(|i| self.free_pool.swap_remove(i))
            .unwrap_or_else(|| vec![Sample::default(); needed].into_boxed_slice())
    }

    fn release_scratch(&mut self, buf: Box<[Sample]>) {
        self.free_pool.push(buf);
    }
}
```

**On `add_node()`**: call `acquire_scratch(n_inputs)` and `acquire_scratch(n_outputs)`, rebuild ExecPlan.

**On `remove_node()`**: call `release_scratch(entry.input_scratch)` and `release_scratch(entry.output_scratch)`, rebuild ExecPlan.

Allocations only happen in `add_node()` when the free list is empty. `run()` never allocates.

---

## Fan-out

When multiple downstream nodes share the same upstream output, each independently copies from `done[j].output_scratch` during its gather phase. This is N memcopies instead of zero-copy.

At 128 samples × 4 bytes = 512 bytes per copy, sequential memory access, this is negligible in practice. If profiling shows fan-out copies are a bottleneck, the unsafe flat-pool approach (see `resonix-graph-buffer-api.md`) can be adopted later with a contained `unsafe` block in the gather step only.

---

## Comparison vs Other Proposals

| | **This doc (safe gather)** | Flat pool (Proposal 1) | SmallVec only (Proposal 3) |
|---|---|---|---|
| **Allocations in run loop** | Zero | Zero | Near-zero (SmallVec ≤8) |
| **Fan-out cost** | N memcopy | Zero copy | N memcopy |
| **Unsafe** | **None** | In graph internals | None |
| **Node deletion** | Free list — clean | Compaction problem | Free list — clean |
| **Trait changes** | None | Breaking | Minor (`block_size`) |
| **Implementation risk** | Low | Medium | Low |

---

## Open Questions

1. **ExecPlan rebuild cost**: structural changes (add/connect/remove) trigger a full ExecPlan rebuild. Is this acceptable, or does hot-swap (atomic pointer swap from another thread) need to be designed now?

2. **External output API**: the `HashMap<PortAddress, Vec<Sample>>` currently passed to `run()` still clones at the graph boundary. Should this become a pre-allocated read-only slab instead?

3. **Fan-in (multiple outputs → one input)**: currently the last writer wins. Should fan-in sum into `input_scratch` instead of copy? This would match Pd's behavior.

4. **SmallVec inline size**: 4 is assumed above. A histogram of port counts across real patches would confirm the right number.

5. **Unconnected input ports**: if `n_inputs` counts only connected ports, the slot indexing in `input_scratch` needs to map port IDs → scratch slot indices. If it counts all declared ports, unconnected slots stay zeroed in scratch. Which is cleaner?

---

## References

- Current run loop: `crates/resonix-graph/src/implementations/graph.rs`
- Current trait: `crates/resonix-graph/src/traits/audio_node.rs`
- Unsafe flat-pool proposals: `ideas/resonix-graph-buffer-api.md`
- Pure Data signal model: `ideas/pd-analysis/PURE_DATA_KEY_FINDINGS.md`
