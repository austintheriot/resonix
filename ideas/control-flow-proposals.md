# Control Flow Architecture Proposals for Resonix

## Background

Resonix currently implements audio flow but no control flow (messages, scalar values, etc.). This document explores how control flow could be integrated, informed by analysis of Pure Data's architecture and Resonix's design constraints.

**Hard constraints:**
- `no_std` — no system clock, no dynamic dispatch via threads
- Zero allocation in `run()`
- Control I/O pushed outside the graph (like audio I/O — caller provides values)
- Must be very performant

**Existing seeds in the codebase:** `ParamNode`, `Data { None, F32, I32, Error }`, and `Node` enum already distinguish audio from param nodes.

---

## Framing: What is "control" in Resonix?

Pure Data conflates three distinct things under "messages":
1. **Parameters** — continuously-valued, block-rate (filter cutoff, gain)
2. **Events** — discrete, timestamped occurrences (note-on, trigger)
3. **Structural routing** — named global send/receive

A key design choice for Resonix: **keep these distinct**. Pd's loose atom typing exists because these three things share the same message type. Separating them enables stronger guarantees and cleaner semantics.

---

## Control Value Shape: Scalar vs. List, and Why Not Multi-Channel

### No multi-channel control

Audio multi-channel is justified because channels are parallel, *homogeneous* streams — left and right in a stereo signal are structurally identical; the same operation applies to both. Control values are almost never like this. `cutoff` and `resonance` are semantically distinct parameters; they should be separate connections, not two "channels" of the same one.

The scenario that makes multi-channel control seem appealing — "drive N identical things simultaneously" — is better handled two other ways:
- **Fan-out**: one control output connected to N control inputs. Already supported by the graph's fan-out mechanism.
- **Lists** (see below): if the N values truly belong together as a compound value (e.g. a 3D position), that's a list, not channels.

Removing `channels` from control port descriptors keeps the model simple: every control connection is either a scalar or a fixed-arity list, and the arity is declared by the port, not the connection.

### Lists via fixed-arity connections

Variable-length lists (like Pd's `t_atom[]`) require heap allocation, which is banned during `run()`. The solution mirrors exactly how audio handles channel count: **declare the arity at `connect()` time**, pre-allocate N f32 values in the buffer pool, and pass `&[f32]` of known length to `process()`. Zero cost, no new infrastructure.

```rust
pub enum ControlArity {
    Scalar,        // exactly 1 f32
    List(usize),   // exactly N f32s, fixed for the connection's lifetime
}
```

An arity mismatch between connected ports is caught at `connect()` — same as a channel count mismatch for audio today. Examples of types that map cleanly to fixed-arity lists:

| Value type               | Arity |
|--------------------------|-------|
| Gain / frequency / etc.  | 1     |
| Vec2 (e.g. pan position) | 2     |
| Vec3 (3D position)       | 3     |
| RGBA color               | 4     |
| ADSR envelope params     | 4     |

The arity is fixed for the lifetime of a connection. Changing the "shape" of a control signal is a structural patch change — it requires a `disconnect()`/`reconnect()`, just like changing channel count on an audio connection would. This is not a meaningful limitation in practice.

**What about strings/symbols?** Pd's `A_SYMBOL` is one of its messiest features, requiring a global interned string table. Resonix should not support string control values initially. If needed, a fixed-capacity `[u8; N]` byte slice pre-allocated at connect time would work, but most musical control is numeric. Skip it until there's a concrete use case.

**What about bang (trigger, no value)?** Model as `ControlArity::Scalar` with a `bool`, or as `Event<()>` in the event queue model (Proposal B) for sub-block timing. No special type needed.

---

## Proposal A: Block-Rate Control Buffers

**Model:** Control connections work exactly like audio connections but at block granularity — a "control buffer" holds one f32 (or a fixed-arity list of f32s) updated once per `run()` cycle. The compiled plan runs control nodes first, then audio nodes.

```rust
pub trait ControlNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[Option<ControlBuffer<'_>>],
        outputs: &mut [Option<ControlBufferMut<'_>>],
    ) -> Result<(), ControlNodeRunError>;
}

// A scalar control connection: &[f32] of length 1
// A list control connection: &[f32] of length N (arity fixed at connect() time)
// Same buffer pool infrastructure as audio — just no block_size multiplier
pub struct ControlBuffer<'a>(&'a [f32]);
```

The compiled plan becomes two phases:
1. Execute all `ControlNode`s in topological order
2. Execute all `AudioNode`s in topological order (existing system)

Cross-domain connections (control → audio, audio → control) are **explicit converter nodes** — no implicit coercion. The graph rejects incompatible port connections.

**Differentiators vs. Pd/Max:**
- Zero implicit type coercion — unlike Pd where floats flow into signals via signalinlets
- Control and audio topologies share the same connection model — same `PortId`, `ConnectionId`, `BufferPool` infrastructure
- Block-rate control is extremely cache-friendly and predictable
- External control I/O works exactly like external audio I/O — caller provides values before `run()`

**Weakness:** No sub-block event timing. A note-on that arrives mid-block is quantized to the block boundary. This is the Max MSP "control rate" tradeoff.

---

## Proposal B: Sample-Accurate Event Queues

**Model:** Each control connection carries a fixed-capacity ring buffer of `(sample_offset: u16, value: T)` pairs. Events have sample-accurate timestamps within the block. No scheduler, no system clock — all timing is relative to the current block.

```rust
pub struct EventQueue<T, const CAP: usize> {
    events: [Option<Event<T>>; CAP],  // fixed-size, no alloc
    head: usize,
    len: usize,
}

pub struct Event<T> {
    pub sample_offset: u16,  // 0..block_size
    pub value: T,
}
```

Pre-allocated at `connect()` time, just like audio buffers. The pool holds `Box<UnsafeCell<EventQueueStorage>>` with a fixed max capacity (configurable per connection).

**Execution model:**
1. External caller pushes events into external input queues (like filling audio input buffers now)
2. `run()` executes nodes in topological order; each node drains its input queues and may push to output queues
3. After `run()`, caller reads events from external output queues

**Differentiators vs. Pd/Max:**
- **Sample-accurate without a scheduler** — no Pd-style clock tick, no drift, no global time. Timing is encoded in the event itself as a sample offset. This is the model used in CLAP and LV2's atom events.
- No dynamic allocation — queue capacity is fixed at connection time
- Strongly typed: `EventQueue<NoteOn, 64>` vs `EventQueue<f32, 16>` — incompatible connections rejected at `connect()`
- Enables sequencers, arpeggiators, etc. that need sub-block precision

**Weakness:** Fixed capacity means potential event dropping. A policy is needed (drop oldest, drop newest, error). The `no_std` constraint means no growable queue during `run()`.

---

## Proposal C: Typed Rate-Tagged Ports

**Model:** Ports carry a **rate tag** — `Audio`, `Block`, or `Event(capacity)` — enforced at `connect()`. The graph validates rate compatibility. Hybrid nodes explicitly declare which ports are at which rate.

```rust
pub enum PortRate {
    Audio,                      // block_size * channels samples per run()
    Block { arity: usize },     // arity f32 values per run() (1 = scalar, N = list)
    Event { cap: u8 },          // up to cap timestamped events per run()
}

pub struct PortDescriptor {
    pub address: PortAddress,
    pub channels: usize,        // meaningful for Audio only; ignored for Block/Event
    pub rate: PortRate,         // NEW
}
```

The compiled plan is a single pass, but each `CompiledStep` knows which of its ports are which rate. The buffer pool gains three buffer types. Rate-conversion nodes (e.g., "sample and hold", "upsample control to audio") are explicit.

**External I/O:** The caller provides typed buckets:
```rust
pub fn run(
    &mut self,
    audio_inputs: &[ExternalAudioBuffer],
    audio_outputs: &mut [ExternalAudioBuffer],
    control_inputs: &[ExternalControlValue],    // block-rate
    control_outputs: &mut [ExternalControlValue],
    event_inputs: &[ExternalEventQueue],
    event_outputs: &mut [ExternalEventQueue],
) -> Result<(), GraphRunError>
```

**Differentiators vs. Pd/Max:**
- **The rate is part of the port type** — something no major patching environment does explicitly. Pd infers it from the `~` naming convention. Max is similar. Resonix makes it structural.
- Users can define nodes with mixed-rate ports (e.g., a filter node with audio I/O + block-rate frequency control + event-rate trigger)
- No "anything" fallback — every port is statically characterized
- Enables the compiler/graph to generate radically different execution strategies per domain

---

## Proposal D: Parameter Overlay via Atomics (Out-of-Band Control)

**Model:** Don't add control flow to the graph at all. Node parameters are exposed as `Arc<Atomic>` handles returned at `add()` time. External code (UI thread, network thread, MIDI handler) writes to these atomics; nodes read them during `process()`.

```rust
pub struct FilterNodeHandle {
    node_handle: NodeHandle<PortDescriptors>,
    pub cutoff: Arc<AtomicF32>,
    pub resonance: Arc<AtomicF32>,
}
```

**Differentiators vs. Pd/Max:**
- Completely sidesteps the control flow problem — graph stays "pure"
- Parameters can be updated at any time from any thread with no synchronization cost in `run()`
- No graph invalidation, no topology change, no compiled plan update
- Works naturally with UI frameworks, OSC, MIDI handlers

**Weakness:** No inter-node control flow. You can't have a node whose output controls another node's parameter without manually wiring up the atomics outside the graph. Significant limitation for generative/algorithmic patches.

---

## Recommendation: Phased Approach

These proposals are not mutually exclusive. A natural progression:

**Phase 1 — Proposal A (Block-Rate Control):** Extend `ParamNode` to have a real `process()` signature with `ControlBuffer` inputs/outputs. Add `PortRate::Block` to port descriptors, validate at `connect()`. The compiled plan runs control before audio. Gives inter-node control flow with zero new infrastructure complexity.

**Phase 2 — Proposal B (Events):** Add `PortRate::Event { cap }` and fixed-capacity event queues. Handles MIDI, triggers, and anything needing sub-block timing. The buffer pool gains a third type.

**Phase 3 (optional) — Proposal D:** Expose parameter atomics as a convenience API on top of Phase 1, so external code can update control values without going through the external graph I/O interface.

---

## Core Differentiators to Lean Into vs. Pd/Max

1. **Rate tags are structural, not naming conventions** — no `~` suffix convention; rate is encoded in `PortDescriptor`
2. **No implicit type coercion** — incompatible ports fail at `connect()`, not silently at runtime
3. **No global symbol namespace** — no `send`/`receive` with string names; all routing is explicit in the graph
4. **Sample-accurate events without a scheduler** — timing is encoded in events as sample offsets, not driven by a global clock tick

The "no scheduler" angle is genuinely novel and plays to Rust's strengths — determinism and predictability without Pd's scheduler complexity (clock drift, tick granularity, thread synchronization).
