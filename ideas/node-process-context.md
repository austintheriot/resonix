# NodeProcessContext Design & Buffer Collection

## Concept

`NodeProcessContext` is a short-lived bridge struct that routes `AudioBuffer` references into `AudioNode::process`. Buffers are long-lived (allocated before DSP runs); the references are borrowed only during each node's processing turn.

The direction is sound, but several concrete issues need resolving before it works.

---

## Problems

### 1. Borrow-checker conflict in `Graph::run`

```rust
// node borrows self.graph_items mutably
let Some(GraphItem::Node(Node::AudioNode(node))) = self.graph_items.get_mut(id) else { ... };

// ...then you need self.buffer_pool — borrow conflict
.map(|connection_id| self.buffer_pool.get(connection_id).unwrap());
```

The compiler can split disjoint struct field borrows, but not when the borrow comes through a HashMap method return value. This is why the current code falls back to `let inputs = []; let mut outputs = [];`.

**Fix**: Two-pass approach — read port addresses (immutable borrow, released), access buffer_pool, then take the mutable borrow again to call `process`.

### 2. `RefMut` guards must outlive the context

`BufferPool` stores `RefCell<AudioBuffer>`. Calling `borrow_mut()` returns a `RefMut<'_, AudioBuffer>` guard. The raw `&mut AudioBuffer` reference is only valid while that guard is alive.

`NodeProcessContext<'a>` storing `&'a mut AudioBuffer` can't safely hold these. The guards must be stored alongside the context.

**Fix**: Change `NodeProcessContext` to hold `Ref<'a, AudioBuffer>` / `RefMut<'a, AudioBuffer>` guards instead of raw references. This couples it to `RefCell`, but `BufferPool` already uses `RefCell`, so the coupling already exists.

```rust
pub struct NodeProcessContext<'a> {
    inputs: [Option<Ref<'a, AudioBuffer>>; PortId::MAX_PORT_ID],
    outputs: [Option<RefMut<'a, AudioBuffer>>; PortId::MAX_PORT_ID],
    block_size: BlockSize,
}
```

### 3. `NodeProcessContext::new` allocates per-call

```rust
inputs: Box::new([None; PortId::MAX_PORT_ID]),         // heap alloc, every block
outputs: Box::new([const { None }; PortId::MAX_PORT_ID]),  // heap alloc, every block
```

Two 256-element box allocations per node per audio block — contradicts the zero-allocation goal.

**Options**:
- Pre-allocate one `NodeProcessContext` on `Graph`, reset it each iteration
- Use stack arrays (remove `Box::new`) — requires the context to stay stack-local

### 4. Inputs and outputs must be separated

The current collector chains them:
```rust
node.output_port_addresses()...chain(node.input_port_addresses()...)
```

Outputs need `borrow_mut()` (written by this node); inputs need `borrow()` (read, written by a predecessor). They must be in separate loops when populating the context.

### 5. Wrong type annotation in `input_buffer`

```rust
.and_then(|inner: &Option<&Box<[f32]>>| inner.as_deref())
```

`and_then` passes the `T` value, not `&T`, so `inner` would be `Option<&Box<[f32]>>`, not `&Option<...>`. Also `as_deref()` on `Option<&Box<[f32]>>` returns `Option<&[f32]>`, which doesn't match the `Option<&AudioBuffer>` return type. Won't compile.

---

## Concrete Approach for Buffer Collection in `run`

```rust
for id in visit_order.iter() {
    // Pass 1: collect port addresses WITHOUT a mutable borrow on the node
    let (output_addresses, input_addresses) = {
        let Some(GraphItem::Node(Node::AudioNode(node))) = self.graph_items.get(id) else {
            return Err(GraphRunError::VisitOrderIncludedNonNodeValue);
        };
        (
            node.output_port_addresses().unwrap_or(&[]).to_owned(),
            node.input_port_addresses().unwrap_or(&[]).to_owned(),
        )
    }; // immutable borrow of graph_items released here

    // Pass 2: build the context using buffer_pool (no graph_items borrow active)
    let mut ctx = NodeProcessContext::new(self.block_size);

    for addr in &output_addresses {
        if let Some(conn_id) = self.port_address_to_connection_id_map.get(addr) {
            if let Some(cell) = self.buffer_pool.get(conn_id) {
                ctx.set_output_buffer(addr.port_id(), cell.borrow_mut());
            }
        }
    }
    for addr in &input_addresses {
        if let Some(conn_id) = self.port_address_to_connection_id_map.get(addr) {
            if let Some(cell) = self.buffer_pool.get(conn_id) {
                ctx.set_input_buffer(addr.port_id(), cell.borrow());
            }
        }
    }

    // Pass 3: now mutably borrow the node to call process
    let Some(GraphItem::Node(Node::AudioNode(node))) = self.graph_items.get_mut(id) else {
        return Err(GraphRunError::VisitOrderIncludedNonNodeValue);
    };
    node.process(&mut ctx)?;
}
```

The two `to_owned()` calls are allocations. To eliminate them, either:
- Store port address lists separately from the node (already partially done via `port_address_to_connection_id_map`)
- Use `RefCell<Box<dyn AudioNode>>` so `graph_items` only needs an immutable borrow to access the node

---

## Key Insight: `PortAddress::port_id()`

`PortAddress::port_id()` already gives you the node-local port index to use as the key in `NodeProcessContext`. `addr.port_id()` is the right argument to `set_input_buffer`/`set_output_buffer` — no guessing needed.

---

## Summary

| Problem | Fix |
|---|---|
| `NodeProcessContext` allocates per-call | Store guards directly or pre-allocate on `Graph` |
| Holds `&mut AudioBuffer` instead of guards | Store `Ref<'a>`/`RefMut<'a>` guards |
| Borrow conflict in `run` | Two-pass: read addresses immutably, release, then access buffer_pool |
| Chains inputs and outputs | Separate loops for `output_port_addresses` vs `input_port_addresses` |
| `AudioNode::process` doesn't take `NodeProcessContext` | Change the trait signature |
