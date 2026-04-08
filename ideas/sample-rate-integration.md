# Sample Rate Integration in Resonix

*Date: 2026-04-08*

Based on the Pure Data architecture analysis (`pd-analysis/SAMPLE_RATE_ARCHITECTURE.md`) and the current Resonix codebase state.

---

## Current State

Sample rate does not exist in Resonix's core graph at all. It is handled externally:

- The CPAL audio driver reports a sample rate (e.g., `audio_output.config().sample_rate.0`)
- Application code uses it manually for math (e.g., `440.0 * PI / sample_rate as f32` in CLI tests)
- The graph receives only `block_size: BlockSize` and `current_time: CurrentTime` per processing tick
- No `SampleRate` primitive, no `SampleRate` field on Graph, no propagation to nodes

This means nodes that depend on sample rate — oscillators, filters, delay lines, envelope generators — cannot be self-contained. They require the caller to pre-compute frequency ratios or time constants and pass them in as parameters. That works at prototype scale but breaks down as the node library grows.

---

## What Pure Data Gets Right (and Wrong)

**Right:**

1. Sample rate is a property of each *signal*, not just a global. `t_signal.s_sr` enables sub-patches to legitimately run at different effective rates.
2. A rate change triggers a full graph rebuild — no hot-swap ambiguity.
3. Resampling is injected *at compile time* into the flat DSP chain, not at runtime — zero overhead per tick.

**Wrong (for Resonix's goals):**

1. Global mutable state (`STUFF->st_dacsr`) is a singleton. Resonix's embeddability goal requires multiple independent graph instances, possibly at different rates.
2. No-std and WASM are incompatible with Pd's global state approach.
3. Pd's graph rebuild is implicit and opaque; Resonix should make rebuilds explicit and observable.

---

## Proposed Design

### 1. Add `SampleRate` as a First-Class Primitive

```rust
// crates/resonix-graph/src/primitives/sample_rate.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleRate(pub u32);

impl SampleRate {
    pub fn as_f32(self) -> f32 { self.0 as f32 }
    pub fn as_f64(self) -> f64 { self.0 as f64 }
}
```

Mirrors `BlockSize` and `CurrentTime` — a typed wrapper around a primitive, zero overhead.

---

### 2. Store `SampleRate` on `Graph`, Not Globally

```rust
pub struct Graph {
    // existing fields...
    block_size: BlockSize,
    sample_rate: SampleRate,   // NEW
}
```

Graph is created with a sample rate:

```rust
Graph::new(block_size: BlockSize, sample_rate: SampleRate) -> Self
```

**Why:** Resonix's embeddability goal requires multiple independent graph instances. A module running in a game at 44100 Hz and a background renderer at 22050 Hz should coexist without interference. Global state makes that impossible.

---

### 3. Pass `SampleRate` into `AudioNode::process`

The `AudioNode` trait's `process` signature becomes:

```rust
fn process<A: AudioBuffer, M: AudioBufferMut>(
    &mut self,
    inputs: &[Option<A>],
    outputs: &mut [Option<M>],
    block_size: BlockSize,
    current_time: CurrentTime,
    sample_rate: SampleRate,   // NEW
) -> Result<(), AudioNodeRunError>;
```

**Why not a `ProcessContext` struct instead?**

The `node-process-context-v2.md` analysis already identified that a `NodeProcessContext` struct may not earn its keep — the complexity is a `Graph::run` concern, not a node concern. However, the five-argument `process` signature is starting to look wide. There are two defensible paths:

**Option A: Keep flat parameters** — consistent with current `block_size` + `current_time` pattern, zero indirection.

**Option B: Introduce a `ProcessContext`** — bundle `block_size`, `current_time`, and `sample_rate` (all graph-level metadata, not signal data) into a single `ProcessContext` value type. Buffers remain separate parameters. This would look like:

```rust
pub struct ProcessContext {
    pub block_size: BlockSize,
    pub current_time: CurrentTime,
    pub sample_rate: SampleRate,
}

fn process<A: AudioBuffer, M: AudioBufferMut>(
    &mut self,
    inputs: &[Option<A>],
    outputs: &mut [Option<M>],
    ctx: ProcessContext,
) -> Result<(), AudioNodeRunError>;
```

`ProcessContext` is a plain value type (Copy), so there is no overhead. It is also forward-compatible: adding `tempo`, `time_signature`, or future graph-level context does not change the `process` signature again. **This option is recommended** given the direction toward more metadata (see future considerations below).

---

### 4. Sample Rate Changes Trigger a Graph Rebuild

When the audio driver changes sample rate (e.g., user switches output device), the application must call:

```rust
graph.set_sample_rate(new_rate: SampleRate);
```

This invalidates the compiled plan:

```rust
pub fn set_sample_rate(&mut self, rate: SampleRate) {
    self.sample_rate = rate;
    self.compiled_plan = None; // force recompile on next run()
}
```

Nodes that cache computed coefficients (e.g., a biquad filter storing pre-computed `a0`, `b1`, etc.) must recompute them. The mechanism: nodes can implement an optional `on_sample_rate_change` hook, **or** simply recompute coefficients every block lazily when they detect `ctx.sample_rate != self.cached_sample_rate`. The lazy approach requires no extra trait method and no notification infrastructure.

**Recommendation:** Start with lazy recompute. No pub/sub, no callbacks. Nodes store a `cached_sample_rate: Option<SampleRate>` and recompute when it changes. This is simple, correct, and testable.

---

### 5. What Resonix Should NOT Copy from Pure Data

**Do not put sample rate in signals/buffers.** Pd stores `s_sr` per signal to enable sub-patches at different rates. That feature is powerful but the complexity cost is significant: every signal allocation carries extra state, resampling nodes must be injected at compile time, and graph rebuild logic must walk canvas hierarchies to compute effective rates.

Resonix does not yet have sub-graphs at all (`node-process-context-v2.md` and the exploration confirm this). The right order is:

1. Add sample rate to the graph (this proposal)
2. Add sub-graph / nested graph support (separate proposal)
3. Add per-sub-graph sample rate override, with automatic resampling (only if there is a concrete use case)

Starting with sample-rate-per-signal before sub-graphs exist is premature and would add complexity to every buffer operation with no user-visible benefit today.

---

## What Nodes Need to Do

A sine oscillator currently has no way to convert a frequency in Hz to a per-sample phase increment. With `SampleRate` in `ProcessContext`:

```rust
// Before: caller must externally compute and pass the increment
// After: oscillator is self-contained
fn process<A, M>(&mut self, _inputs, outputs, ctx: ProcessContext) {
    let phase_increment = self.frequency_hz / ctx.sample_rate.as_f32();
    for sample in outputs[0].channel_mut(0) {
        *sample = (self.phase * 2.0 * PI).sin();
        self.phase = (self.phase + phase_increment) % 1.0;
    }
}
```

Filter nodes (biquad, SVF) compute their coefficients from `frequency / sample_rate`. Delay lines compute their buffer lengths from `delay_seconds * sample_rate`. Envelopes compute their per-sample increment from `attack_seconds * sample_rate`. All of these are currently impossible to implement cleanly.

---

## Migration Path

1. Add `SampleRate` primitive to `resonix-graph/src/primitives/`
2. Add `sample_rate: SampleRate` field to `Graph` struct; update constructor(s)
3. Add `Graph::sample_rate()` accessor and `Graph::set_sample_rate()` mutator
4. Introduce `ProcessContext { block_size, current_time, sample_rate }` (value type, Copy)
5. Update `AudioNode::process` signature to use `ProcessContext`
6. Update `Graph::run` to construct and pass `ProcessContext` to each node
7. Update all existing node implementations (currently simple: `SineNode`, `ConstantNode`, `MultiplyNode`, `PassthroughNode` — none use sample rate yet, so changes are mechanical)
8. Update `AudioNode` documentation

Each step compiles and passes tests independently. No existing node behavior changes — this is purely additive until nodes begin using sample rate.

---

## Open Questions for You

1. **`ProcessContext` vs flat parameters**: Do you prefer bundling `block_size + current_time + sample_rate` into `ProcessContext`, or keeping them as separate arguments? The bundle is more extensible; flat args are more explicit.

2. **Lazy vs. notification-based coefficient recomputation**: Should nodes detect sample rate changes themselves (lazy, via a stored `Option<SampleRate>`), or should the graph call an `on_config_change` hook before `process`? Lazy is simpler; explicit hooks make the protocol more self-documenting.

3. **Sub-graph scope**: When sub-graphs eventually exist, should they inherit the parent's sample rate by default, or always require an explicit sample rate? Inheritance is ergonomic; explicit is less surprising.

4. **Integer vs. float sample rate storage**: `SampleRate(u32)` matches hardware reality (44100, 48000, 96000 are always integer). But some algorithms prefer `f32` or `f64` to avoid repeated casts in hot paths. The `as_f32()`/`as_f64()` accessors paper over this, but if most node math ends up as floats, should the primitive store `f32` instead?
