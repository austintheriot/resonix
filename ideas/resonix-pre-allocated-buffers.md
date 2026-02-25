# Pre-Allocated Buffers: AudioNode Trait Design

*Date: 2026-02-24*

This document outlines how to modify the `AudioNode` trait to support pre-allocated buffer pools for zero-copy audio processing.

---

## Current Trait (v2)

**Location:** `crates/resonix-graph/src/traits/audio_node.rs`

```rust
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[&DataBlock],
        outputs: &mut [&mut DataBlock],
    ) -> Result<(), AudioNodeRunError>;
}
```

### Problems

1. **`DataBlock` abstraction hides buffer details** - Wraps actual audio data in enum
2. **No explicit block size** - Nodes don't know how many samples to process
3. **Enables copying** - Easy to clone `DataBlock`, defeating zero-copy goal
4. **No control-rate support** - Can't distinguish audio buffers from scalar parameters

---

## Proposed Trait (Zero-Copy)

```rust
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    /// Process audio with pre-allocated buffers
    ///
    /// # Arguments
    /// * `inputs` - Slices to pre-allocated input buffers (read-only)
    /// * `outputs` - Slices to pre-allocated output buffers (write directly here)
    /// * `block_size` - Number of samples to process (d buffer capacity)
    ///
    /// # Guarantees
    /// - Buffers are pre-allocated and reused across calls (zero-copy)
    /// - All buffers are SIMD-aligned (32-byte alignment)
    /// - `block_size` may be less than buffer capacity (process only requested range)
    /// - If a node connects to itself, outputs won't be in inputs (no aliasing)
    ///
    /// # Performance Notes
    /// - Write directly to output buffers (no intermediate allocations)
    /// - Use SIMD when possible (buffers are aligned)
    /// - Process exactly `block_size` samples, ignore remainder
    fn process(
        &mut self,
        inputs: &[&[f32]],          // Raw slices (zero-copy)
        outputs: &mut [&mut [f32]], // Raw slices (write directly)
        block_size: usize,          // Explicit sample count
    ) -> Result<(), AudioNodeRunError>;
}
```

### Key Changes

#### 1. Raw Buffer Slices
```rust
// Before
inputs: &[&DataBlock]
outputs: &mut [&mut DataBlock]

// After
inputs: &[&[f32]]
outputs: &mut [&mut [f32]]
```

**Benefits:**
-  Zero-copy (write directly to pre-allocated output)
-  SIMD-friendly (contiguous f32 data)
-  No enum matching overhead
-  No accidental cloning

#### 2. Explicit Block Size
```rust
block_size: usize
```

**Why needed:**
- Buffers allocated as power-of-2 sizes (64, 128, 256, etc.)
- CPAL may request fewer samples than buffer capacity
- Process only requested range: `outputs[0][0..block_size]`
- Enables variable block sizes across platforms

#### 3. Documentation Guarantees
- Buffers pre-allocated (no allocation in `process`)
- SIMD-aligned (32-byte for AVX)
- No aliasing (Rust borrow checker enforces)

---

## Example Implementations

### Simple Multiply Node

```rust
pub struct MultiplyNode;

impl AudioNode for MultiplyNode {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError> {
        // Expect 2 inputs, 1 output
        let in1 = inputs[0];
        let in2 = inputs[1];
        let out = &mut outputs[0];

        // Process samples (writes directly to pre-allocated buffer)
        for i in 0..block_size {
            out[i] = in1[i] * in2[i];
        }

        Ok(())
    }
}
```

### SIMD-Optimized Node

```rust
use std::simd::{f32x8, SimdFloat};

pub struct SimdMultiplyNode;

impl AudioNode for SimdMultiplyNode {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError> {
        let in1 = inputs[0];
        let in2 = inputs[1];
        let out = &mut outputs[0];

        let chunks = block_size / 8;
        let remainder = block_size % 8;

        // Process 8 samples at a time
        for i in 0..chunks {
            let idx = i * 8;
            let a = f32x8::from_slice(&in1[idx..]);
            let b = f32x8::from_slice(&in2[idx..]);
            let result = a * b;
            result.copy_to_slice(&mut out[idx..]);
        }

        // Handle remainder samples
        let start = chunks * 8;
        for i in start..block_size {
            out[i] = in1[i] * in2[i];
        }

        Ok(())
    }
}
```

### Node with Internal State

```rust
pub struct LowPassFilter {
    cutoff: f32,
    state: f32, // Previous output sample
}

impl AudioNode for LowPassFilter {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError> {
        let input = inputs[0];
        let output = &mut outputs[0];

        let alpha = self.cutoff;

        for i in 0..block_size {
            self.state = self.state + alpha * (input[i] - self.state);
            output[i] = self.state;
        }

        Ok(())
    }
}
```

---

## Advanced: Separate Audio and Control Inputs

For nodes that need both audio-rate buffers and control-rate parameters:

```rust
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        // Audio-rate inputs (48kHz buffers)
        audio_inputs: &[&[f32]],

        // Control-rate inputs (scalar values, updated per-frame)
        control_inputs: &[f32],

        // Audio-rate outputs (48kHz buffers)
        audio_outputs: &mut [&mut [f32]],

        // Number of samples to process
        block_size: usize,
    ) -> Result<(), AudioNodeRunError>;
}
```

### Example: Oscillator with Frequency Control

```rust
pub struct Oscillator {
    phase: f32,
    sample_rate: f32,
}

impl AudioNode for Oscillator {
    fn process(
        &mut self,
        _audio_inputs: &[&[f32]],
        control_inputs: &[f32],      // control_inputs[0] = frequency
        audio_outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError> {
        let frequency = control_inputs[0];
        let output = &mut audio_outputs[0];

        let phase_increment = 2.0 * std::f32::consts::PI * frequency / self.sample_rate;

        for i in 0..block_size {
            output[i] = self.phase.sin();
            self.phase += phase_increment;

            // Wrap phase to avoid floating point precision issues
            if self.phase > 2.0 * std::f32::consts::PI {
                self.phase -= 2.0 * std::f32::consts::PI;
            }
        }

        Ok(())
    }
}
```

---

## Migration Strategy

### Option 1: Clean Break (Recommended)

Just change the trait and update all node implementations:

```rust
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError>;
}
```

**Pros:**
- Clean, no legacy baggage
- Compiler finds all places to update
- Forced to think about zero-copy

**Cons:**
- Breaks all existing nodes
- Must update everything at once

---

### Option 2: Dual Methods (Gradual Migration)

Support both old and new methods temporarily:

```rust
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    /// New optimized method (implement this in new code)
    fn process_buffers(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError> {
        // Default: call legacy method with conversion
        let input_blocks: Vec<DataBlock> = inputs
            .iter()
            .map(|slice| DataBlock::Audio {
                samples: (*slice)[0..block_size].to_vec()
            })
            .collect();

        let input_refs: Vec<&DataBlock> = input_blocks.iter().collect();

        let mut output_blocks: Vec<DataBlock> = (0..outputs.len())
            .map(|_| DataBlock::Audio {
                samples: vec![0.0; block_size]
            })
            .collect();

        let mut output_refs: Vec<&mut DataBlock> = output_blocks.iter_mut().collect();

        self.process_legacy(&input_refs, &mut output_refs)?;

        // Copy results back
        for (i, block) in output_blocks.iter().enumerate() {
            if let DataBlock::Audio { samples } = block {
                outputs[i][0..block_size].copy_from_slice(samples);
            }
        }

        Ok(())
    }

    /// Legacy method (deprecated)
    #[deprecated(note = "Implement process_buffers() for better performance")]
    fn process_legacy(
        &mut self,
        inputs: &[&DataBlock],
        outputs: &mut [&mut DataBlock],
    ) -> Result<(), AudioNodeRunError> {
        // Default: panic (must implement one or the other)
        unimplemented!("Must implement either process_buffers() or process_legacy()")
    }
}
```

**Usage:**

```rust
// Old nodes still work (with conversion overhead)
impl AudioNode for OldMultiplyNode {
    fn process_legacy(
        &mut self,
        inputs: &[&DataBlock],
        outputs: &mut [&mut DataBlock],
    ) -> Result<(), AudioNodeRunError> {
        // Old implementation
    }
}

// New nodes get zero-copy performance
impl AudioNode for NewMultiplyNode {
    fn process_buffers(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError> {
        // New implementation
    }
}
```

**Pros:**
- Gradual migration
- Old code keeps working (with warning)
- Can update nodes incrementally

**Cons:**
- More complex trait
- Temporary conversion overhead
- Need to clean up eventually

---

### Option 3: Feature Flag

Keep both versions behind feature flags:

```toml
[features]
default = ["legacy"]
legacy = []
optimized = ["buffer-pool"]
```

```rust
#[cfg(feature = "legacy")]
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[&DataBlock],
        outputs: &mut [&mut DataBlock],
    ) -> Result<(), AudioNodeRunError>;
}

#[cfg(feature = "optimized")]
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError>;
}
```

**Pros:**
- Can benchmark both versions
- Users choose when to migrate
- Clear separation

**Cons:**
- Duplicate code
- Conditional compilation complexity
- Need to maintain both

---

## Integration with Graph

The `Graph` struct needs to provide these pre-allocated buffers:

```rust
impl Graph {
    pub fn process_block(&mut self, block_size: usize) -> Result<(), ProcessError> {
        for &node_id in &self.visit_order {
            let node = &mut self.nodes[node_id];

            // Build input slices from buffer pool (no allocation!)
            let input_slices: SmallVec<[&[f32]; 4]> = self
                .get_input_buffer_ids(node_id)
                .iter()
                .map(|buffer_id| {
                    self.buffer_pool.get_slice(*buffer_id, block_size)
                })
                .collect();

            // Build output slices from buffer pool (no allocation!)
            let mut output_slices: SmallVec<[&mut [f32]; 4]> = self
                .get_output_buffer_ids(node_id)
                .iter()
                .map(|buffer_id| {
                    self.buffer_pool.get_slice_mut(*buffer_id, block_size)
                })
                .collect();

            // Process writes directly to pre-allocated buffers (zero-copy!)
            node.process(&input_slices, &mut output_slices, block_size)?;
        }

        Ok(())
    }
}
```

---

## Comparison to Pure Data

Pure Data uses a similar approach:

```c
// Pure Data's perform routine signature
typedef t_int *(*t_perfroutine)(t_int *w);

// Example: plus_perform
t_int *plus_perform(t_int *w) {
    t_sample *in1 = (t_sample *)(w[1]);  // Pointer to input buffer
    t_sample *in2 = (t_sample *)(w[2]);
    t_sample *out = (t_sample *)(w[3]);  // Pointer to output buffer
    int n = (int)(w[4]);                  // Block size

    while (n--) *out++ = *in1++ + *in2++;
    return (w+5);
}
```

**Similarities:**
- Raw pointers to pre-allocated buffers
- Explicit block size parameter
- Zero-copy (write directly to output)

**Differences:**
- Pure Data uses raw pointers (unsafe)
- Resonix uses Rust slices (safe, bounds-checked)
- Pure Data has linked-list DSP chain
- Resonix has pre-computed visit order

---

## Performance Impact

### Before (Current v2)

```rust
// Pseudo-code showing cloning
for node_id in &visit_order {
    let node = &mut self.nodes[node_id];

    // Clone data from connections into inputs
    for (port, connection_id) in input_connections {
        let data = self.connections_data.get(&connection_id);
        self.inputs[port] = data.clone();  // L ALLOCATION #1
    }

    // Process
    node.process(&self.inputs, &mut self.outputs)?;

    // Clone outputs to connection storage
    for (port, connection_id) in output_connections {
        self.connections_data.insert(
            connection_id,
            self.outputs[port].clone()  // L ALLOCATION #2
        );
    }
}
```

**Cost per frame (100-node graph, 150 connections):**
- 150 clones of `DataBlock` per frame
- At 48kHz: 7,200,000 clones/second
- Each clone: allocation + memcpy
- **~112,500 heap operations/second**

### After (Pre-Allocated Buffers)

```rust
for &node_id in &visit_order {
    let node = &mut self.nodes[node_id];

    // Get slices to pre-allocated buffers (no allocation!)
    let inputs = self.get_input_slices(node_id);
    let outputs = self.get_output_slices_mut(node_id);

    // Process writes directly to output buffers (zero-copy!)
    node.process(&inputs, &mut outputs, block_size)?;
}
```

**Cost per frame:**
- 0 allocations
- 0 clones
- Only pointer arithmetic (nanoseconds)
- **10-100x faster**

---

## Recommendation

**Start with Option 1 (Clean Break):**

1. Change the trait signature now
2. Update all ~3 existing nodes (ConstantNode, MultiplyNode, OutputNode)
3. No legacy baggage
4. Clean foundation for future growth

You only have a few nodes currently, so migration cost is minimal. The performance and simplicity gains are worth it.

---

## Next Steps

1.  Update `AudioNode` trait signature
2.  Update existing node implementations
3.  Implement `BufferPool` (see `architecture-proposals.md`)
4.  Update `Graph::run()` to use buffer pool
5.  Add benchmarks to verify zero-copy performance
6.  Run under Miri to validate safety

---

## References

- [Architecture Proposals](./resonix-architecture-proposals.md) - Full buffer pool design
- [Architecture Critique](./resonix-architecture-critique.md) - Performance analysis
- Pure Data source: `d_ugen.c` - DSP chain execution
- Current trait: `crates/resonix-graph/src/traits/audio_node.rs`
