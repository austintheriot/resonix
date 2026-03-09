# Control Flow Architecture — Resonix v3

## What Changed from v2

Two substantive revisions:

1. **`Atom<'arena>` is replaced by a `RawAtom` / `Atom<'a>` split**, mirroring the existing `RawAudioBuffer` / `AudioBuffer<'a>` pattern. Variable-size atom payloads are stored as `NonNull` pointers without a lifetime parameter, eliminating the arena-tied lifetime from queue storage and enabling FFI-friendly representation. The safe `Atom<'a>` view with a lifetime appears only at `process()` call sites, not in stored state.

2. **The atom kind set is explicitly open.** The existing `Data` enum (`None`, `F32`, `I32`, `Error`) is not the final list. The v3 design uses a `u32` discriminant for atom kinds, making the format stable and extensible without ABI breakage. A `Raw` escape hatch supports custom or future kinds (musical primitives, GPU buffers, etc.) before they become first-class variants.

---

## The Atom Representation

The pattern follows `RawAudioBuffer` / `AudioBuffer<'a>` exactly:

```rust
/// Stored in message queues — no lifetime, repr(C), FFI-stable.
/// Variable-size payloads (Symbol, List, Raw) point into the arena,
/// graph-internal storage, or caller-supplied storage.
#[repr(C)]
pub struct RawAtom {
    pub kind: AtomKind,
    pub payload: RawAtomPayload,
}

#[repr(u32)]
#[non_exhaustive]
pub enum AtomKind {
    Bang    = 0,
    Float   = 1,
    Int     = 2,
    Symbol  = 3,  // UTF-8 bytes
    List    = 4,  // array of RawAtom
    Raw     = 5,  // custom/future: (type_tag: u32, bytes)
    // Future: Note, Duration, MidiEvent, ...
}

#[repr(C)]
pub union RawAtomPayload {
    pub float:  f32,
    pub int:    i32,
    pub bytes:  RawAtomBytes,  // used by Symbol, List, Raw
}

/// Thin pointer + length. For Symbol: UTF-8 bytes.
/// For List: array of RawAtom. For Raw: opaque bytes preceded by u32 type tag.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawAtomBytes {
    pub ptr: NonNull<u8>,
    pub len: usize,
}
```

`Option<RawAtom>` uses the null-pointer niche of `ptr` inside `RawAtomBytes` — same optimization as `Option<RawAudioBuffer>`.

The safe view used inside `process()`:

```rust
/// Ergonomic Rust-facing atom. The lifetime 'a covers the validity window
/// of any pointer-backed variants (Symbol, List, Raw).
pub enum Atom<'a> {
    Bang,
    Float(f32),
    Int(i32),
    Symbol(&'a str),
    List(&'a [RawAtom]),
    Raw(u32, &'a [u8]),  // type_tag, bytes
}
```

Converting `&'a RawAtom` → `Atom<'a>` is a safe operation (pointer read with lifetime tied to the caller's borrow). Converting `Atom<'a>` → `RawAtom` for enqueuing is also safe but requires the caller to guarantee the pointer remains valid for the intended scope (see below).

---

## Atom Validity Model: Three Scopes

Without a lifetime in `RawAtom`, pointer validity becomes a documented invariant rather than a compiler-checked one — the same trade made for audio buffer pointers in the compiled plan.

There are three validity scopes:

| Scope | Backing storage | Valid until | Safe to cache across runs? |
|-------|----------------|-------------|---------------------------|
| **Graph-scope** | Node fields, graph-owned string/list storage | Graph is dropped | Yes |
| **Run-scope** | Message arena | Start of next `run()` (arena reset) | No |
| **Caller-scope** | Caller-supplied external message buffer | `run()` returns | No |

Run-scope and caller-scope are equivalent from a node's perspective: both are valid for exactly one `run()` invocation. Nodes must not store `RawAtom`s with run/caller-scope pointers in fields that outlive `process()`. Nodes may store graph-scope `RawAtom`s freely (e.g. a constant symbol node holds its `RawAtom` in a field and emits it every run with zero copy into the arena).

Miri coverage of the message-passing path is expected to catch violations of these invariants, the same way it catches audio buffer violations today.

---

## Atom Extensibility

The `AtomKind` discriminant is `u32`. New first-class kinds (e.g. `Note`, `Duration`, `Pitch`, `MidiEvent`) are added by reserving new values — no ABI change to existing atoms.

The `Raw(u32, &[u8])` variant is the escape hatch for:
- Domain-specific types before they graduate to first-class
- External/plugin-defined types
- Large binary payloads (e.g. GPU texture handles, image data)

The `u32` type tag inside `Raw` is user-defined. A registry of well-known tags (analogous to LV2 URID) can be layered on top without changing the core format.

**What's explicitly not in the initial kind set:**

Symbols in Pd serve double duty as both string data and message selectors (method dispatch names). In Resonix, `Symbol` is purely data — a UTF-8 string. Method dispatch is not part of the message model (see "No Global Symbol Table" below). A node receives a `List` or an arbitrary sequence of atoms and interprets them however it wants. There is no selector-based dispatch at the graph level.

---

## Message Arena

A pre-allocated bump allocator owned by the graph, reset at the start of each `run()`. Provides run-scope storage for dynamically constructed symbols and lists.

```rust
pub struct MessageArena<const N: usize = 65536> {
    storage: [u8; N],  // stack-allocatable for compile-time graph compilation
    used: usize,
}

impl<const N: usize> MessageArena<N> {
    pub fn reset(&mut self) { self.used = 0; }

    /// Allocate a UTF-8 string. Returns None if arena is full.
    pub fn alloc_str(&mut self, s: &str) -> Option<&str> { /* bump */ }

    /// Allocate a RawAtom slice. Returns None if arena is full.
    pub fn alloc_atoms(&mut self, n: usize) -> Option<&mut [RawAtom]> { /* bump */ }
}
```

The const generic size satisfies the compile-time graph compilation future goal: with `N` known at compile time, `MessageArena<N>` can live on the stack or in a static with zero heap involvement. For runtime-configured graphs, `Box<MessageArena<N>>` with a large `N` works the same way as today.

If an allocation fails (arena full), `run()` returns an error. No silent data loss. Arena capacity is set at graph construction time and documented as a tunable for the caller.

**What the arena is not:** it is not a message pool that gets recycled per-message. It is a per-run scratch pad. Nodes that need persistent storage across runs allocate it in their fields at construction time (graph-scope).

---

## Message Queues Per Connection

Each message connection carries a pre-allocated ring buffer of `RawAtom` values. Capacity is configured at `connect()` time with a default (e.g. 256).

```rust
pub struct MessageQueue {
    slots: Box<[Option<RawAtom>]>,  // allocated at connect() time
    head: usize,
    len: usize,
    overflow: OverflowPolicy,
}

pub enum OverflowPolicy {
    DropOldest,  // default — matches Pd scheduler behavior
    DropNewest,
    Error,
}
```

The capacity bound is the number of *pending, undelivered* atoms between two adjacent nodes within a single `run()`. Because the compiled plan executes nodes in topological order, sender always runs before receiver; in the common case the receiver drains the queue before it can fill. High-throughput connections (sequencers, dense MIDI streams) set higher capacity at `connect()` time. This is an explicit memory budget decision, not a silent limitation.

Fan-out of a message connection reuses the existing buffer-pool fan-out mechanism: all consumers of the same output port read from the same `MessageQueue`. This is structurally identical to audio fan-out.

---

## Port Model

```rust
pub enum PortRate {
    /// block_size × channels samples per run().
    Audio,
    /// A ring buffer of RawAtom per run(). Capacity set at connect() time.
    Message,
}

pub struct PortDescriptor {
    pub address: PortAddress,
    pub rate: PortRate,
    /// Meaningful for Audio ports only; 0 for Message ports.
    pub channels: usize,
}
```

There is no separate "scalar control" rate. A scalar is a one-atom message. A block-rate parameter is a message connection that typically carries one `Float` atom per run. The queue abstraction handles all cases uniformly.

Cross-domain connections (`Audio` ↔ `Message`) are rejected at `connect()`. Conversion is explicit via converter nodes (envelope follower, sample-and-hold, etc.).

---

## Execution Model

Two-phase compiled plan, identical structure to the existing audio plan:

```
Phase 1: Execute all MessageNodes in topological order
Phase 2: Execute all AudioNodes in topological order
```

```rust
pub trait MessageNode {
    fn process(
        &mut self,
        inputs: &[&MessageQueue],
        outputs: &mut [&mut MessageQueue],
        arena: &mut MessageArena,
        block_size: BlockSize,
    ) -> Result<(), MessageNodeError>;
}
```

Nodes read `RawAtom`s from input queues, converting to `Atom<'_>` via a safe accessor. They write `RawAtom`s to output queues, constructing variable-size payloads in `arena` if needed. Scalar atoms (`Bang`, `Float`, `Int`) require no arena allocation.

`block_size` is available for nodes that reason about timing (BPM conversion, rate calculation) without introducing a scheduler.

**Block-rate as a deliberate design choice:**

PRIORITIES.md lists "differentiation between message/event timing and audio timing" as a known weak spot in Max/Pd — it causes class of bugs where a message fires a sample early or late due to scheduler granularity. Block-rate message passing eliminates this: all messages take effect at block boundaries. The execution order is fully deterministic. This is not a concession; it is the fix for a documented Pd failure mode.

---

## External I/O

```rust
pub fn run(
    &mut self,
    audio_inputs:    &[ExternalAudioBuffer],
    audio_outputs:   &mut [ExternalAudioBuffer],
    message_inputs:  &[&MessageQueue],      // caller populates before run()
    message_outputs: &mut [&mut MessageQueue], // caller reads after run()
) -> Result<(), GraphRunError>
```

External message inputs carry caller-scope atoms. The caller owns the queue storage; the graph reads from it during Phase 1 and does not hold references after `run()` returns. This is structurally identical to how external audio I/O works today.

---

## On Symbols: No Global Interning Table

Pd's symbol interning (`gensym`) is a process-wide mutable hash table. This is hostile to:
- Wasm (global mutable state between instances is shared memory — wrong behavior)
- Multi-graph embedding (two independent graphs would share a namespace)
- `no_std` (no heap allocator for the table at module load time)

In Resonix, `Symbol` is `&str` — UTF-8 bytes in the arena (run-scope), in node fields (graph-scope), or in caller storage (caller-scope). Equality is value-based, not identity-based. There is no global table.

The use case that motivates Pd symbols — "route this message to the named object" — is handled in Resonix by explicit graph connections. Named global routing (`send`/`receive`) is not part of the graph model. If a higher-level layer wants name-based routing, it can be built on top of the explicit connection graph without embedding it in the core atom type.

---

## Comparison with Pure Data

| Aspect | Pure Data | Resonix v3 |
|--------|-----------|------------|
| Stored atom type | `t_atom { t_atomtype; union word; }` — with ptr into interned table | `RawAtom { AtomKind: u32; RawAtomPayload }` — repr(C), NonNull |
| Safe view at use site | None (raw union everywhere) | `Atom<'a>` enum with lifetime |
| Atom kind extensibility | Fixed C enum, recompile to add | `u32` discriminant, stable ABI |
| Arbitrary-size lists | `(int argc, t_atom *argv)` on the C stack | `&[RawAtom]` into arena (run-scope) or node storage (graph-scope) |
| Strings/symbols | Globally interned, O(1) identity compare | `&str` bytes, no global table, value equality |
| Messages per connection/tick | Unlimited (linked-list traversal) | Configurable ring buffer, explicit capacity |
| Type coercion | Implicit (float↔symbol↔list) | None — incompatible port rates rejected at `connect()` |
| Rate encoding | `~` naming convention | Structural `PortRate` field in `PortDescriptor` |
| Sub-block timing | Full scheduler, sample-accurate | Not supported; block boundaries only |
| Allocation in dispatch | `malloc` in list construction | Bump arena, zero system allocator calls |
| FFI representation | C struct, native | `repr(C)` with `u32` discriminant and union — directly exposable |

---

## Phased Implementation

**Phase 1 — Port rate tagging:**
Add `PortRate` to `PortDescriptor`. Validate rate compatibility at `connect()`. No message passing yet — just the structural foundation and connection-time enforcement.

**Phase 2 — Scalar message passing:**
Implement `MessageQueue` with `RawAtom` storage. Support `Bang`, `Float`, `Int` (no arena needed — all inline payloads). Add `MessageNode` trait and two-phase compiled plan. Wire up `ExternalMessageSlot`. This covers the majority of parameter-passing use cases.

**Phase 3 — Variable-size atoms:**
Add `MessageArena`. Enable `Symbol` and `List` variants. Enable `Raw` escape hatch. Nodes that need strings or lists allocate from `arena` in `process()`. Existing scalar-only nodes are unaffected and require no changes.
