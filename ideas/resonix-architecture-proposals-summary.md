# Resonix Architecture Summary

*Date: 2026-02-24*
*Full proposals: [resonix-architecture-proposals.md](./resonix-architecture-proposals.md)*

---

## Vision

Resonix differentiates from Max/Pure Data through:
- **Compile-time safety** (Rust type system catches errors before runtime)
- **WASM-native** (runs in browser without plugins)
- **Radically embeddable** (bare-metal → desktop → web)
- **Modular** (use just the core, or full GUI stack)
- **Collaborative-ready** (CRDT-based real-time co-editing)

---

## Core Problems to Solve

Based on Pure Data analysis and current Resonix v2 issues:

1. **Data cloning kills performance** - Current code clones `Data` at every connection (~112,500 allocations/sec in 100-node graph)
2. **Audio callback panics** - `.unwrap()` at `cpal_audio_output.rs:49` crashes on underrun
3. **No parameter updates** - Can't change node parameters while audio running
4. **No I/O isolation** - Core logic mixed with platform-specific code
5. **No type safety** - Can connect incompatible port types at runtime

---

## 5 Key Proposals

### 1. Three-Tier Architecture

```
GUI Layer (resonix-gui)           ← WASM + egui, visual patching
       ↓
I/O Layer (resonix-audio)         ← CPAL, Web Audio, error recovery
       ↓
Pure Core (resonix-core, no_std)  ← Zero-allocation graph execution
```

**Benefits:**
- Core works on bare-metal (no OS, no allocator)
- Each tier independently testable
- Swap I/O backends without changing core
- GUI optional (can use as library)

---

### 2. Pre-Allocated Buffer Pool

**Problem:** Current code clones `Data` at every connection point.

**Solution:** Allocate all buffers once, pass buffer IDs (not contents):

```rust
pub struct BufferPool {
    storage: Vec<AlignedBuffer>,           // Pre-allocated (power-of-2 sizes)
    free_lists: [Vec<BufferId>; 8],        // Per-size free lists
    ref_counts: Vec<AtomicU16>,            // Zero-copy sharing
}

impl Graph {
    fn process_block(&mut self, block_size: usize) -> Result<(), ProcessError> {
        for &node_id in &self.visit_order {
            // Get buffer IDs (not contents!)
            let input_buffers = self.get_input_buffer_slices(node_id);
            let output_buffers = self.get_output_buffer_slices_mut(node_id);

            // Process writes directly to output buffers (zero-copy)
            node.process(input_buffers, output_buffers, block_size)?;
        }
        Ok(())
    }
}
```

**Impact:**
- ✅ Zero allocations in audio thread (Pure Data style)
- ✅ 10x+ performance improvement
- ✅ Deterministic timing (no GC pauses)
- ✅ SIMD-friendly (aligned, contiguous buffers)

---

### 3. Robust Error Handling

**Problem:** Audio callback panics on ring buffer underrun.

**Solution:** Graceful fallback, never panic:

```rust
let mut last_sample = S::EQUILIBRIUM;
let error_stats = Arc::new(ErrorStats::default());

let mut next_value = move || {
    match consumer.read() {
        Ok(sample) => {
            last_sample = sample;
            sample
        }
        Err(_) => {
            // Underrun: log error, return last sample (don't crash!)
            error_stats.underruns.fetch_add(1, Ordering::Relaxed);
            last_sample
        }
    }
};
```

**Impact:**
- ✅ Never crashes audio (Pure Data approach)
- ✅ Reports errors to UI
- ✅ Configurable latency (1-100ms)
- ✅ Production-ready reliability

---

### 4. Thread-Safe Parameter Updates

**Problem:** No mechanism to change node parameters while audio running.

**Solution:** Lock-free message queue:

```rust
pub struct ParamUpdateQueue {
    tx: Sender<ParamUpdate>,  // UI thread writes here
    rx: Receiver<ParamUpdate>, // Audio thread reads here
}

impl Graph {
    pub fn process_block(&mut self) -> Result<(), ProcessError> {
        // Apply parameter changes at frame boundary (lock-free)
        while let Ok(update) = self.param_queue.rx.try_recv() {
            self.apply_param_update(update);
        }

        // Then process audio
        for node_id in &self.visit_order {
            // ...
        }
    }
}
```

**Impact:**
- ✅ Safe real-time automation
- ✅ No locks in audio thread
- ✅ Changes applied at frame boundaries (no artifacts)

---

### 5. Compile-Time Type Safety

**Problem:** Can connect incompatible ports, discover at runtime.

**Solution:** Type-level verification:

```rust
pub struct Port<T: PortType, D: Direction> {
    _phantom: PhantomData<(T, D)>,
}

impl Graph {
    pub fn connect_typed<T: PortType>(
        &mut self,
        source: Port<T, Output>,
        dest: Port<T, Input>,
    ) -> Result<(), ConnectionError> {
        // Type system enforces compatibility!
        self.connect_internal(source.into(), dest.into())
    }
}

// This compiles:
graph.connect_typed(osc.audio_out, filter.audio_in)?;

// This fails at compile time:
// graph.connect_typed(osc.freq_control, filter.audio_in)?;
//                     ^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^
//                     Control port      Audio port - TYPE MISMATCH!
```

**Impact:**
- ✅ Catch connection errors at compile time
- ✅ No runtime type checks
- ✅ Better IDE autocomplete

---

## Additional Enhancements

### SIMD Optimization
Process 4-8 samples simultaneously → 4-8x speedup on vectorizable operations

### Multi-Rate Processing
Control-rate parameters (~750Hz) vs audio-rate signals (48kHz) → reduce CPU

### Testing Strategy
- **Snapshot tests** - Bit-exact audio output comparison
- **Fuzz tests** - Find edge cases automatically
- **Miri** - Detect undefined behavior

---

## Implementation Roadmap

### Phase 1: Core Foundation (4-6 weeks)
- Fix `Graph::run()` compilation errors
- Implement `BufferPool` with zero-copy routing
- Add comprehensive tests + Miri validation
- **Goal:** 10x performance improvement

### Phase 2: I/O Robustness (2-3 weeks)
- Remove `.unwrap()` panics
- Add error tracking + reporting
- Implement `ParamUpdateQueue`
- **Goal:** Production-ready audio I/O

### Phase 3: Type Safety & Optimization (3-4 weeks)
- Typed ports (`Port<Audio, Input>`)
- Constant folding + dead code elimination
- SIMD for core nodes
- **Goal:** Compile-time guarantees + 20% speedup

### Phase 4: GUI & WASM (4-6 weeks)
- egui node editor
- Web Audio backend
- Real-time visualizations
- CRDT collaboration
- **Goal:** Browser-based patching

**Total:** 14-19 weeks to full stack

---

## Migration Path

Don't break existing code:

```rust
impl Graph {
    #[deprecated(note = "Use run_optimized()")]
    pub fn run_legacy(&mut self, ...) { /* current implementation */ }

    pub fn run_optimized(&mut self, ...) { /* new zero-copy */ }
}
```

Feature flags:
```toml
[features]
default = ["legacy"]
optimized = ["buffer-pool", "simd"]
```

Benchmark both:
```bash
cargo bench --features=legacy
cargo bench --features=optimized
```

---

## Key Learnings from Pure Data

After 30 years of production use, Pure Data's architecture teaches:

1. **Zero per-frame allocation** - All buffers pre-allocated at graph compile time
2. **Never crash audio thread** - Graceful fallback on underruns
3. **Separate signal/message domains** - Control-rate vs audio-rate processing
4. **Lock-free parameter updates** - Bounded queue processed at frame boundaries
5. **Power-of-2 buffer sizes** - Efficient allocation + SIMD alignment

---

## Open Questions

1. **Minimum target platform?** (32KB RAM embedded? 256KB? 1MB?)
2. **Fixed or variable block size?** (Pd=64, WASM=128)
3. **Support graph hot-swapping?** (changing graph while audio running)
4. **WASM threading model?** (AudioWorklet + SharedArrayBuffer?)
5. **Error handling philosophy?** (Panic / Result / callback per tier?)

---

## References

- Full proposals: [resonix-architecture-proposals.md](./resonix-architecture-proposals.md)
- Pure Data analysis: [pd-analysis/](./pd-analysis/)
- Architecture critique: [resonix-architecture-critique.md](./resonix-architecture-critique.md)
- Project priorities: [GitHub issue #3](https://github.com/austintheriot/resonix/issues/3)

---

## Next Steps

1. **Review proposals** - Gather feedback on approach
2. **Prototype buffer pool** - Validate zero-copy performance
3. **Fix audio callback** - Remove panic, test underrun handling
4. **Benchmark current code** - Establish performance baseline
5. **Phase 1 kickoff** - Start core foundation work

---

*This architecture balances Pure Data's proven reliability with modern Rust capabilities to create a uniquely embeddable, type-safe, and collaborative audio framework.*
