# Pure Data Architecture: Key Findings for Resonix

## Executive Summary

Pure Data's 25-year-old architecture provides proven patterns for real-time audio processing. The system separates concerns into distinct layers: pre-allocated signal buffers, a compiled DSP chain, and platform-agnostic I/O abstraction. Key architectural innovations include power-of-2 buffer binning, reference-counted signal sharing, and ahead-of-time DSP graph compilation.

---

## Critical Architectural Decisions

### 1. Zero Per-Block Allocation
Pure Data's core design principle: **The audio callback must never allocate memory**.

**Evidence**: 
- Signal buffers allocated at graph compile time (`signal_new()`)
- All reuse pools populated before DSP starts
- Per-block execution only iterates pre-built chain
- Cleanup only happens when DSP is disabled

**Implication for Resonix**: 
Block processing callbacks should operate on pre-allocated state. Any dynamic graph changes must happen outside the realtime thread.

### 2. Power-of-2 Buffer Sizes
All signal buffers are sized to the nearest power-of-2 above the requested size.

**Why**:
- Eliminates heap fragmentation over long sessions
- Enables O(1) pool lookup: `freelist[log2(size)]`
- Trades modest memory overhead for deterministic behavior
- Exactly 34 size buckets covers practical range

**Math**:
```
request = 100 samples
log2(100) = 6.6, round up = 7
allocate = 2^7 = 128 samples
waste = 28% per buffer (acceptable)
```

**For Resonix**: Consider same approach; provides predictable memory profile.

### 3. The DSP Chain: Compiled, Linear Execution
Instead of walking a graph at runtime, Pd compiles a flat sequence of function pointers.

**Execution Model**:
```
dsp_chain = [
  func1_ptr, arg1, arg2, arg3,
  func2_ptr, arg1, arg2,
  ...,
  dsp_done_ptr
]

// Execute
for (ip = dsp_chain; ip; )
    ip = (*(t_perfroutine)(*ip))(ip);
```

**Advantages**:
- Cache-friendly: linear memory access
- No dynamic dispatch overhead
- Deterministic execution order
- Easy to profile/optimize

**Disadvantages**:
- Must recompile on every graph change
- Complex graph changes have latency
- Can't add/remove operations mid-block

**For Resonix**: Evaluate whether compiled chains are necessary or if dynamic dispatch is acceptable given heterogeneous block sizes.

### 4. Borrowed Signals: Transparent Buffer Sharing
A signal can "borrow" its buffer from another signal via reference counting.

**Three Signal Types**:
1. **Owned**: Manages its own allocated buffer
2. **Borrowed**: Points to another signal's buffer, increments refcount
3. **Scalar**: Points to a single persistent value

**Pattern**:
```c
// Signal A owns a buffer
sig_A = signal_new(64, 1, 48000, NULL);

// Signal B borrows from A
sig_B = signal_new(0, 1, 48000, NULL);  // length=0 → borrowed
signal_setborrowed(sig_B, sig_A);        // sig_A->refcount++

// When done
signal_dereference(sig_A);   // refcount--, if 0 → reusable
```

**For Resonix**: Enables efficient multi-tap sharing without copying. Critical for objects like `sig-cast~` or multi-output nodes.

### 5. Reference Counting with Deferred Cleanup
Signals aren't freed when refcount reaches zero; they're marked "reusable" and added to freelists.

**Lifecycle**:
- Graph compile: allocate/reclaim signals
- During DSP: signals are "in use" (refcount > 0)
- When object output drops to zero refs: `signal_makereusable()`
- Next compile: reclaim from freelist or allocate

**Safety Property**: Once DSP starts, memory is stable. No allocator calls during audio processing.

**For Resonix**: Simple explicit cleanup on graph changes is probably better than deferred reuse, but the deferred pattern handles complex object destruction gracefully.

---

## I/O Architecture: The Key Isolation Pattern

### Global I/O Buffers
```c
STUFF->st_soundin   // Pre-allocated: channels * 64 samples
STUFF->st_soundout  // Pre-allocated: channels * 64 samples
```

These buffers are touched in exactly two places:
1. **adc~**: Copies hardware input → patch signals
2. **dac~**: Sums patch signals → hardware output

**Critical insight**: dac~ *adds* to the output buffer (using `plus_perform`), not replaces it.

### Separation of Domains

| Domain | Character | Timing |
|--------|-----------|--------|
| **Message** | Event-driven | Sample-accurate |
| **Signal** | Block-processed | 64-sample chunks |

Objects implement both. Example:
- adc~/dac~ (signal domain): Register via `dsp_add()`
- volume control (message domain): Receive float messages
- Same object can do both: update control at block boundaries

**For Resonix**: This dual-domain pattern is powerful. Control messages can change object state, and changes take effect at the next block boundary.

---

## Modularity Patterns

### 1. Abstract I/O Layer
Pd supports 7+ audio APIs (JACK, ALSA, PortAudio, AudioUnit, WinMM, etc.) with a uniform interface:

```c
api_open_audio(inchans, outchans, rate, soundin, soundout, ...);
api_close_audio();
api_send_dacs();
api_getdevs();
```

Selected at runtime via configuration. Core Pd code is API-agnostic.

**For Resonix**: Plugin architecture benefit: support multiple backends (RtAudio, JUCE, PulseAudio) with pluggable implementations.

### 2. External Plugin System
- Objects export `*_setup()` function
- Runtime dynamic loading with version checking
- Architecture-specific binaries (`.l_amd64`, `.d_arm64`, etc.)
- Opaque API: externals access via function pointers, not direct struct access

**Stability Strategy**: Internals can change; external API is stable.

### 3. Sized Allocation Tracking
Every malloc/free pair includes explicit size:

```c
getbytes(size)           // malloc
freebytes(ptr, size)     // free with size verification
resizebytes(ptr, old, new)
```

Enables:
- Leak detection (sum of allocations vs freed)
- Memory profiling
- Debug allocation hooks

**For Resonix**: Essential for long-running services. Implement similar tracking.

---

## Multi-Rate Processing: The `block~` Pattern

For subgraphs with different block sizes, `block~` implements:

1. **Prolog**: Checks phase; skips block if not time to run
2. **Block content**: Normal DSP code
3. **Epilog**: Handles reblocking (upsampling/downsampling)

**Phase Counter**:
```
if (phase % period != 0) skip entire block
else { run block; phase = (phase + 1) % period }
```

This enables:
- Hierarchical time scales
- Efficient oversampling/undersampling
- Nested block structures

**For Resonix**: Support block-level timing, but consider whether `block~`-style multiplexing is necessary vs. simpler block-size negotiation.

---

## Critical Limitations to Avoid

### 1. Circular Signal Dependencies
Pd has **no cycle detection**. Circular graphs cause:
- Infinite loops in compiler
- Silent crashes during execution
- User's responsibility to avoid

**For Resonix**: Add explicit cycle detection in graph validation.

### 2. Live Graph Modification
Changing the patch during DSP requires full recompilation. Pd requires turning off DSP:

```
dsp off → modify graph → recompile → dsp on
```

**For Resonix**: Support live editing? Requires:
- Atomic swap of DSP chains
- Careful state transfer between old/new chains
- Potential audio glitches if not handled perfectly

### 3. Variable Block Sizes
If upstream and downstream blocks differ, Pd auto-reblocks with overhead. Better to standardize.

**For Resonix**: Consider mandating uniform block size across plugin chains.

---

## Performance Considerations

### Cache Behavior
The linear DSP chain is excellent for CPU cache:
- Instruction prefetching works
- Data flows through L1 cache
- No branch prediction surprises (clear termination)

### Latency
Fixed block-size architecture:
- DSP latency = block size / sample rate
- Pd typical: 64 samples @ 48kHz = 1.33ms
- Low latency possible with smaller blocks (trade-off: overhead)

### Memory Scaling
With power-of-2 binning:
- Memory usage grows logarithmically
- 100 signal objects ≈ 500KB (rough)
- 10,000 signal objects ≈ 5MB
- Fragmentation stable over session lifetime

---

## Recommendations for Resonix Architecture

### Must Implement
1. ✅ Pre-allocated I/O buffers (channels × block size)
2. ✅ Power-of-2 buffer sizing with per-size freelists
3. ✅ Reference counting for shared signals
4. ✅ DSP graph compilation to operation list
5. ✅ Sized allocation tracking
6. ✅ Separate message and signal domains

### Should Consider
1. ❓ Lock-free ringbuffer for control→audio thread communication
2. ❓ Cycle detection in graph validation
3. ❓ Atomic DSP chain swaps for live editing
4. ❓ Explicit block-size negotiation vs. auto-reblocking

### Probably Skip
1. ❌ Automatic reblocking (too expensive)
2. ❌ Deferred signal cleanup (explicit is clearer)
3. ❌ Generic external plugin system (if not cross-platform)

---

## Code References for Deep Dives

| Feature | File | Lines | Key Function |
|---------|------|-------|--------------|
| Signal allocation | d_ugen.c | 511-567 | `signal_new()` |
| DSP chain build | d_ugen.c | 365-401 | `dsp_add()` |
| Chain execution | d_ugen.c | 403-411 | `dsp_tick()` |
| Buffer reuse | d_ugen.c | 457-502 | `signal_makereusable()` |
| I/O setup | s_audio.c | 82-131 | `sys_setchsr()` |
| adc~/dac~ | d_dac.c | 45-162 | `adc_dsp()`, `dac_dsp()` |
| Graph compile | d_ugen.c | 744+ | `ugen_start_graph()` |
| block~ logic | d_ugen.c | 130-356 | `block_new()`, prolog/epilog |
| External loading | s_loader.c | 76-200 | Dynamic linking |
| Class dispatch | m_imp.h | 37-70 | `t_class` structure |

---

## Summary

Pure Data's architecture is optimized for:
- **Stability**: 25 years without major redesigns
- **Predictability**: Pre-allocated memory, compiled graphs
- **Flexibility**: Message+signal domains, abstract I/O
- **Portability**: Platform abstraction layers, external plugins

For Resonix, the most critical patterns to adopt are:
1. No allocation in audio callbacks
2. Power-of-2 buffer sizing
3. Reference-counted signal sharing
4. Compiled DSP chains

Doing these well will provide the foundation for reliable, efficient real-time audio processing.
