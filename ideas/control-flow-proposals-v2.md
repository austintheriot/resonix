# Control Flow Architecture — Resonix v2

## What Changed from v1

User feedback that reshapes the design:

1. **Arbitrary-size lists and strings matter from day one** — fixed-arity connections are not sufficient
2. **Nodes must handle many messages per run()** — not just one value per block
3. **Sample-accurate control is not a priority** — block-rate is fine
4. **Out-of-band parameter updates are already solved** — `Rc`/`Arc` + interior mutability; no need to design for this in the graph

This eliminates Proposals B (sample-accurate events) and D (out-of-band atomics) from v1. Proposal A (block-rate control buffers) survives but is significantly revised: the fixed-arity list restriction is dropped, strings are supported, and each connection carries a *queue* of messages rather than a single value.

---

## Background: What Pure Data Gets Right (and Wrong)

After studying Pd's source, the relevant decisions for Resonix are:

**What Pd gets right:**
- **Atoms** — a small discriminated union (`Bang | Float | Symbol | List`) that covers nearly every musical control use case. Lists are `(int argc, t_atom *argv)` — arbitrary length, no fixed arity.
- **No fixed message count limit** — inlets receive however many messages arrive during a scheduler tick; Pd's stack-overflow guard is a depth limit (1000 recursive hops), not a throughput limit.
- **Symbols are cheap** — interned globally, so `symbol("cutoff") == symbol("cutoff")` by identity.

**What Pd gets wrong (or just old):**
- **Global symbol table** — a process-wide hash table; namespace collisions in large patches are real.
- **Type coercion everywhere** — Pd will silently convert floats to symbols and vice versa. In Resonix, incompatible port types fail at `connect()`.
- **Sample-accurate messages via a scheduler** — elegant but complex; adds latency jitter and drift concerns. Not needed for block-rate use.
- **Malloc during message dispatch** — Pd allocates when building lists in transit. Resonix cannot.

**What Max/MSP adds over Pd:**
- `mc.` (multi-channel) audio, which Resonix already handles.
- Dictionary/JSON objects — overkill for audio DSP.

**What VCV Rack does:**
- Ports are fixed at 1 float (CV) or audio. Inter-module "messages" happen entirely out of band (Rack's own event system). No message-passing model in the graph at all — fine for rack-style UI but limiting for algorithmic composition.

**LV2 Atom / CLAP events:**
- Both use a *forge* pattern: a pre-allocated buffer that acts as a bump allocator. Atoms (typed, variable-size blobs) are written into the forge. All references are stable for the duration of the block. This is the cleanest no-std, zero-allocation approach to arbitrary-size messages.

**The synthesis for Resonix:** adopt Pd's atom semantics with LV2's arena/forge allocation strategy.

---

## The Atom Type

```rust
/// A typed message value. `'arena` is the per-run arena lifetime.
pub enum Atom<'arena> {
    Bang,
    Float(f32),
    Int(i32),
    Symbol(&'arena str),        // UTF-8 slice into the run arena
    List(&'arena [Atom<'arena>]), // atom slice into the run arena
}
```

This covers every Pd message type except `Pointer` (not needed) and `$N` substitution (a patch-language feature, not a runtime type). The `'arena` lifetime ensures that `Symbol` and `List` contents are not freed during `run()` — the arena lives for exactly one `run()` invocation.

**Why `Symbol` instead of just `List(&[u8])`?**

Symbols are names: `"note-on"`, `"cutoff"`, `"bang"`. In Pd they're interned; comparison is O(1). Resonix doesn't need a global interning table, but treating symbols as a distinct type signals intent and allows routing logic (`if msg.selector == "note-on"`) without allocating. A `Symbol` in Resonix is simply a `&str` pointing into the arena — a byte copy at send time, but no owned heap allocation.

**What about nested lists and heterogeneous types?**

A `List` is a `&[Atom]` — it can contain any mix of `Float`, `Int`, `Symbol`, and further nested `List`s, all via arena allocation. This is more expressive than Pd's flat `t_atom[]` (which doesn't support nested lists).

---

## Arbitrary-Size Payloads: The Message Arena

The zero-allocation constraint bans `Vec`, `String`, and friends inside `run()`. But "zero allocation" means *no calls to the system allocator* — it does **not** mean "no variable-size data". The solution is a **bump allocator** over a pre-allocated slab.

```rust
/// A per-run scratch allocator. Allocated once, reset on each run().
pub struct MessageArena {
    storage: Box<[u8]>,  // pre-allocated, e.g. 64 KiB
    used: usize,
}

impl MessageArena {
    pub fn reset(&mut self) {
        self.used = 0;
    }

    pub fn alloc_str(&mut self, s: &str) -> Option<&str> { /* bump */ }
    pub fn alloc_atoms<'a>(&'a mut self, n: usize) -> Option<&'a mut [Atom<'a>]> { /* bump */ }
}
```

The arena is owned by the graph and reset at the start of every `run()`. All `Atom::Symbol(&str)` and `Atom::List(&[Atom])` references are slices into this arena — valid for `'arena`, which is tied to the `run()` call, and dead after it returns.

**Capacity:** The arena is sized at graph construction time. The caller specifies it (default: 64 KiB). If an allocation would overflow, `run()` returns an error — no silent data loss. In practice, 64 KiB is enormous for control data: a 128-element list of floats is 512 bytes; thousands of MIDI-style messages fit easily.

This is the same model used by:
- **LV2 Atom Forge** (`lv2_atom_forge_set_buffer`)
- **CLAP event buffers** (fixed-size, caller-owned)
- Game engine "frame allocators" (reset every frame)

---

## Message Queues: Many Messages Per run()

In v1, a control connection carried *one* value per block (a `&[f32]` of fixed arity). This is wrong for musical use: a MIDI cable can deliver 128 note events in a single scheduler tick. A pitch-detection node might emit one float per zero-crossing — potentially dozens per block.

**Each message connection carries a ring buffer of `Atom` values.** Capacity is configured at `connect()` time, not at port-declaration time.

```rust
pub struct MessageQueue<'arena> {
    // Pre-allocated at connect() time. Content references 'arena.
    slots: Box<[Option<Atom<'arena>>]>,
    head: usize,
    len: usize,
}
```

The capacity is a `connect()`-time parameter with a sensible default (e.g. 256). This is not a hard ceiling on musical expressiveness — it's the number of *pending, unread* messages that can stack up between one node and the next within a single `run()`. In practice, topological ordering means the sender runs before the receiver; slots are consumed before any overflow is possible.

**On overflow:** configurable per-connection policy: drop-oldest (default), drop-newest, or error. This is the same tradeoff LV2 and CLAP make. Drop-oldest matches Pd's behavior (scheduler processes in order; old events are stale).

**What "arbitrary number of messages" means in practice:**

The ring buffer capacity is the per-`run()` limit, not a lifetime limit. If a sequencer emits 64 notes per block, set its output queue capacity to 128. There's no architectural reason to keep it small. Capacity costs memory at `connect()` time, not at `run()` time. The user controls this tradeoff explicitly.

---

## Port Model: Audio vs. Message

Two port rates, no more:

```rust
pub enum PortRate {
    /// block_size * channels samples per run().
    Audio,
    /// A queue of typed Atom values per run(). Capacity set at connect().
    Message,
}

pub struct PortDescriptor {
    pub address: PortAddress,
    pub rate: PortRate,
    /// For Audio ports: channel count. Ignored for Message ports.
    pub channels: usize,
}
```

**Why not a separate "scalar control" rate?** Because a single `Atom::Float(f32)` in a queue *is* a scalar. There's no need for a distinct scalar port type — it's just a queue that happens to usually contain one float. Nodes that process block-rate parameters receive them as a one-element queue. The queue abstraction unifies all message shapes.

**Cross-domain connections** (Audio ↔ Message) are rejected at `connect()`. Conversion nodes (e.g., "envelope follower" producing a float message from audio, or "sample-and-hold" writing an audio signal from a control trigger) exist as explicit graph nodes, same as in Pd.

---

## Execution Model

The compiled plan executes in two passes, as in v1:

```
Phase 1: Execute all MessageNodes in topological order
Phase 2: Execute all AudioNodes in topological order (existing system)
```

A `MessageNode` receives queued atoms on its inputs and enqueues atoms on its outputs:

```rust
pub trait MessageNode {
    fn process(
        &mut self,
        inputs: &[MessageInput<'_>],   // each is a drained queue of Atoms
        outputs: &mut [MessageOutput<'_>], // each is an appendable queue
        arena: &mut MessageArena,      // for allocating Symbol/List payloads
        block_size: BlockSize,
    ) -> Result<(), MessageNodeError>;
}

pub struct MessageInput<'arena> {
    /// Drain this iterator to read incoming messages.
    pub atoms: &'arena [Atom<'arena>],
}

pub struct MessageOutput<'arena> {
    pub fn push(&mut self, atom: Atom<'arena>, arena: &mut MessageArena) -> Result<(), QueueFull>;
}
```

Nodes have access to `arena` to construct complex payloads (strings, lists) when building output messages. Simple scalars (`Atom::Float`, `Atom::Bang`) require no arena allocation.

**Block size in message context:** Message nodes receive `block_size` but are not required to do anything with it. It's available for nodes that want to reason about timing (e.g., a BPM-to-samples converter) without introducing a scheduler.

---

## External I/O

The caller interface gains message queues alongside audio buffers:

```rust
pub fn run(
    &mut self,
    audio_inputs: &[ExternalAudioBuffer],
    audio_outputs: &mut [ExternalAudioBuffer],
    message_inputs: &[ExternalMessageSlot],    // NEW
    message_outputs: &mut [ExternalMessageSlot], // NEW
) -> Result<(), GraphRunError>
```

`ExternalMessageSlot` is a reference to a message queue the caller has populated before `run()` (for inputs) or reads after `run()` (for outputs). Same pattern as audio I/O today — the graph doesn't own external data, the caller does.

This is how MIDI, OSC, LFO values, sequencer output, and UI parameter changes enter and exit the graph without out-of-band threading tricks.

---

## Comparison with Pure Data

| Aspect | Pure Data | Resonix v2 |
|--------|-----------|------------|
| Atom types | `Bang`, `Float`, `Symbol`, `List`, `Pointer` | `Bang`, `Float`, `Int`, `Symbol`, `List` |
| Arbitrary-size lists | `(argc, t_atom *)` — unlimited, stack-allocated | `&[Atom]` in per-run arena — bounded by arena size |
| String/symbol support | Global interning table (`gensym`) | `&str` into per-run arena; no global table |
| Messages per connection/tick | Unlimited (linked list traversal) | Configurable ring buffer (default 256) |
| Type coercion | Implicit everywhere | None — incompatible ports fail at `connect()` |
| Rate encoding | `~` naming convention | Structural `PortRate` field |
| Global namespace | `send`/`receive` with string names | No global namespace; all routing explicit |
| Sub-block timing | Full scheduler with sample-accurate offsets | Not supported; block-rate only |
| Allocation model | `malloc` in message dispatch | Bump arena reset per `run()` |
| Out-of-band updates | Not supported in graph | Callers use `Rc`/`Arc` directly on nodes |

---

## Differentiators to Lean Into

1. **The rate is structural, not a naming convention** — `PortRate::Message` vs `PortRate::Audio` in `PortDescriptor`. No `~` suffix. The type system enforces correct connections.

2. **Typed atoms, no coercion** — `Atom::Float` and `Atom::Symbol` are distinct and incompatible. Nodes declare what they accept; the graph rejects mismatches at `connect()`.

3. **Arbitrary-size without allocation** — the arena provides true variable-size messages (arbitrary strings, nested lists) while honoring the zero-alloc-in-`run()` invariant. This is more capable than Pd's flat atom arrays, which don't support nested structure.

4. **Many messages, explicit capacity** — connection capacity is an explicit graph parameter, not an implicit scheduler property. You control the memory budget; you control overflow behavior.

5. **No scheduler, no drift** — block-rate message passing eliminates Pd's tick granularity and clock drift issues. Timing is coarse but deterministic.

---

## Phased Implementation

**Phase 1 — Port rate tagging and connection validation:**
Add `PortRate` to `PortDescriptor`. Add `rate_compatibility_check()` at `connect()`. No message passing yet — just infrastructure.

**Phase 2 — Block-rate scalar messages:**
Implement `MessageQueue` with `Atom::Bang | Atom::Float | Atom::Int` support. No arena yet — scalars don't need it. Add `MessageNode` trait. Run message phase before audio phase in compiled plan. Wire up `ExternalMessageSlot`.

**Phase 3 — Arena and variable-size atoms:**
Add `MessageArena`. Enable `Atom::Symbol` and `Atom::List`. Nodes that need strings or lists opt in by taking `arena` as a parameter. Existing scalar-only nodes are unaffected.
