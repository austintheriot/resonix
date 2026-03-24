# Architectural Recommendations: Resonix vs. RNBO

These recommendations come from a detailed comparison of the Resonix codebase against the RNBO WebAudio runtime (v1.4.3). RNBO is the reference implementation Resonix is modelled after, so gaps in Resonix relative to RNBO are the focus.

Ordered roughly by impact.

---

## 1. Add a Parameter System — biggest gap

RNBO's parameter system is the primary control surface for a patch. It is not bolted on; it is a first-class concept at every layer. Resonix currently has no equivalent.

RNBO distinguishes:
- **Number / Enum / Bang parameters** — control-rate, settable from outside the graph at any time
- **Signal parameters** — audio-rate, flow through the graph as additional input channels

Resonix's `MultiplyNode` stores `left_operand_value` and `right_operand_value` directly on the node struct, using the last-seen audio sample as a held value. That's a reasonable DSP trick, but it conflates signal processing with parameter state. The consequence: there's no way for external code to set a parameter by name, query its range, or automate it over time.

**Recommendation:** Introduce a `Parameter` abstraction at the graph level, separate from audio connections:

```rust
pub struct ParameterDescriptor {
    pub id: ParameterId,
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

pub trait HasParameters {
    fn parameter_descriptors() -> &'static [ParameterDescriptor];
    fn set_parameter(&mut self, id: ParameterId, value: f32);
    fn get_parameter(&self, id: ParameterId) -> f32;
}
```

Nodes that expose parameters implement `HasParameters`. The graph collects these at `add_audio_node` time (similar to how it already collects port descriptors). A `NodeHandle` gains a `parameters()` method.

**Critical:** In RNBO, parameters are **scheduled with a timestamp** (`scheduleParameterEvent(target, time, value)`). This is what makes sample-accurate automation possible. Even if Resonix doesn't implement full sample-accurate scheduling immediately, the API should accept a time argument from day one — it's very painful to retrofit.

---

## 2. Introduce a Typed Event/Message System

RNBO has a clean inport/outport message system alongside parameters. Nodes can send and receive typed messages (number, list, bang) on named ports. This is how patch-level logic flows that isn't audio signals — it's what makes RNBO patches feel like Max/MSP patches rather than just a fixed DSP graph.

Resonix has no equivalent. Right now the only way for data to flow between nodes is as audio samples through `Input`/`Output` connections.

**Recommendation:** Add message ports as a distinct port direction, separate from audio:

```rust
pub enum PortKind {
    Audio,   // current behavior — block-sized sample buffers
    Message, // new — carries typed payloads, not audio samples
}

pub enum Message {
    Number(f32),
    List(Vec<f32>),
    Bang,
}
```

Message ports don't need buffers or block-synchronized delivery. They're driven by a per-block queue. Nodes that have message ports implement a second trait:

```rust
pub trait HasMessagePorts {
    fn receive_message(&mut self, port: PortId, msg: Message);
    fn drain_outgoing_messages(&mut self) -> impl Iterator<Item = (PortId, Message)>;
}
```

This enables RNBO-equivalent `[number~]`, `[param]`, `[send]`/`[receive]`-style objects.

---

## 3. The `ExternalInput`/`ExternalOutput` port direction is right, but the API surfaces too much

In RNBO, the `InputNode`/`OutputNode` pattern (special nodes that bridge external I/O into the graph) is exactly what Resonix has. That's correct.

The friction in Resonix is that external buffers are referenced by `ExternalConnectionId` at `run()` time, and users have to track these IDs themselves from the `NodeHandle`. Compare to RNBO where the `AudioWorkletNode` just has `inputs[0]` and `outputs[0]` — the mapping is completely opaque to the caller.

**Recommendation:** Consider whether the caller ever needs to know about `ExternalConnectionId` directly, or whether the graph could expose a simpler interface that maps node identity (which users already have from `NodeHandle`) directly to I/O buffers:

```rust
graph.run_with_io(
    &[(input_node_handle.node_id(), &input_samples)],
    &mut [(output_node_handle.node_id(), &mut output_samples)],
)
```

`ExternalConnectionId` becomes a private implementation detail rather than a public API surface.

---

## 4. The `PortDescriptors` type parameter on `NodeHandle` is ergonomic friction

Currently every type that touches a node must be generic over `P: DescribePorts`. This means you can't store a heterogeneous collection of `NodeHandle`s, and code that just wants a node ID has to carry the type parameter along.

```rust
// Today: can't do this — different P types
let handles: Vec<NodeHandle<_>> = vec![handle_a, handle_b];
```

RNBO avoids this by erasing the type at the boundary — `BaseDevice` exposes `parameters[]` and `inports[]` as plain arrays populated from the descriptor at init time. The descriptor type never escapes.

**Recommendation:** Erase `P` on `NodeHandle` after construction:

```rust
pub struct NodeHandle {
    pub node_id: NodeId,
    pub inputs:  Box<[PortAddress]>,  // materialized at add_audio_node time
    pub outputs: Box<[PortAddress]>,
    // parameters, message ports, etc. alongside
}
```

Node-specific named accessors (e.g. `handle.left_operand_input_address()`) can live on a separate type that the node author returns from `get_port_descriptors()`, but users who just want to wire things up work with plain `PortAddress` values and don't carry the generic.

---

## 5. Separate graph topology from execution plan for threading

Resonix already does this implicitly with `compiled_plan: Option<Vec<CompiledStep>>` — that's good. But the separation isn't enforced by types: the `Graph` struct owns both the mutable topology and the compiled plan together, which means they can't live on separate threads.

RNBO has a clean separation here: `BaseEngine` owns metadata and scheduling on the main thread; `WASMHelper` + the `AudioWorkletProcessor` own execution on the audio thread. Topology changes are communicated via a typed message protocol over a `MessagePort`.

**Recommendation:** As Resonix moves toward multi-threading (necessary for a real-time audio tool), formalize this split:

```rust
pub struct GraphTopology { /* petgraph, port maps, buffer pool */ }
pub struct CompiledGraph  { /* Vec<CompiledStep>, immutable during run */ }

impl GraphTopology {
    pub fn compile(&self) -> CompiledGraph { ... }
}

impl CompiledGraph {
    pub fn run(&mut self, ...) { ... }
}
```

`GraphTopology` lives on the control thread. `CompiledGraph` is `Send` and lives on the audio thread. When the topology changes, a new `CompiledGraph` is compiled and swapped in via a lock-free channel (e.g. `triple-buffer` crate). This is the standard real-time audio pattern and maps directly to how RNBO's `ProcessorEngine` proxy works.

---

## 6. Move ID assignment into the graph, not the node

Resonix requires `id_generator: &mut G` at node construction time. RNBO assigns IDs when nodes join the graph, not at construction — nodes don't know or care about their IDs before they are added.

The current approach has a subtle problem: if you construct a node outside a graph (for testing, or to hold as a pending addition), it consumes an ID from whatever generator you give it. That ID may conflict with the graph's own generator if they're not the same instance.

**Recommendation:** Remove `id_generator` from node constructors entirely. Nodes are constructed with only their DSP parameters. The graph assigns a `NodeId` when `add_audio_node` is called:

```rust
// Before
let node = MultiplyNode::new(&mut id_generator);

// After
let node = MultiplyNode::new(); // just DSP state
let handle = graph.add_audio_node(node); // graph assigns the ID
```

This also eliminates the need for `TestIdGenerator` in unit tests of individual nodes — those tests only care about DSP behavior, not IDs.

---

## 7. Make control-rate vs. audio-rate an explicit port distinction

RNBO distinguishes `ParameterType.Signal` (audio-rate, updated per sample via a full buffer) from `ParameterType.Number` (control-rate, a single value set between blocks). Resonix currently only has audio-rate connections.

The `MultiplyNode.left_operand_value` held value is essentially a control-rate input that happens to live inside the node struct. That works for simple cases but doesn't scale — every node that wants a settable constant has to reinvent this pattern independently.

**Recommendation:** Add a `ControlInput` port direction alongside `Input`/`Output`. Control inputs receive a single `f32` per block rather than a full buffer. The graph passes these as scalars, with no buffer allocation needed:

```rust
pub enum PortAddressDirection {
    Input,          // existing: block-sized audio buffer
    Output,         // existing: block-sized audio buffer
    ControlInput,   // new: single f32 per block
    ExternalInput,  // existing
    ExternalOutput, // existing
}
```

This maps naturally to RNBO's non-signal parameter inputs and cleanly replaces the held-value pattern in nodes like `MultiplyNode`.

---

## Summary

| Gap | Priority | RNBO Analogue |
|-----|----------|---------------|
| Parameter system with scheduling | High | `ParameterEvent`, `scheduleParameterEvent` |
| Message ports (inport/outport) | High | `MessageEvent`, `sendNumMessage` etc. |
| Simpler external I/O API | Medium | `device.node` + Web Audio `.connect()` |
| Type-erased `NodeHandle` | Medium | `BaseDevice.parameters[]` / `inports[]` |
| Topology/execution split for threading | Medium | `ProcessorEngine` proxy pattern |
| Move ID assignment into graph | Low | Graph-assigned IDs in RNBO |
| Control-rate port kind | Low | `ParameterType.Number` vs `Signal` |

The graph execution core — compiled plan, buffer pool, SCC ordering, unsafe pointer strategy — is solid and production-quality. The gaps are almost entirely at the *interface* layer: how external code controls the graph, and how nodes communicate with each other beyond raw audio signal connections.
