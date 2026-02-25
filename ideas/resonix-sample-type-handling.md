# Sample Type Handling: Making AudioNode Compatible with CPAL

*Date: 2026-02-24*

This document explains how to make the `AudioNode` trait compatible with CPAL's multiple sample types (f32, i16, u16, f64) while keeping the core pure and type-agnostic.

---

## The Problem

**CPAL supports multiple sample formats:**
```rust
pub trait Sample: Copy + Clone + ... {
    // Implemented for: f32, i16, u16, f64
}
```

**Your current audio output is generic:**
```rust
pub struct CpalAudioOutput<S: Sample> {
    producer: Producer<S>,
    stream: Stream,
    // ...
}
```

**But your AudioNode trait wants f32 slices:**
```rust
pub trait AudioNode {
    fn process(
        &mut self,
        inputs: &[&[f32]],  // ← Hardcoded to f32
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError>;
}
```

**Question:** How do we reconcile the two?

---

## How Pure Data Solves This

### 1. Internal Format is Always Normalized Float

Pure Data internally uses `t_sample` (typedef for float or double):

```c
// m_pd.h
#if PD_FLOATSIZE == 32
  typedef float t_sample;
#elif PD_FLOATSIZE == 64
  typedef double t_sample;
#endif
```

**All DSP processing uses `t_sample`** in the range [-1.0, 1.0].

### 2. Conversion Happens ONLY at I/O Boundaries

Each audio backend converts between hardware format and internal format:

**ALSA backend** (s_audio_alsa.c):
```c
// Output: t_sample (float) → hardware format
if (alsa_outdev[iodev].a_sampwidth == 4) {  // 32-bit
    t_sample s1 = *fp2 * INT32_MAX;
    ((t_alsa_sample32 *)alsa_snd_buf)[j] = CLIP32(s1);
}
else if (alsa_outdev[iodev].a_sampwidth == 2) {  // 16-bit
    int s = *fp2 * 32767.;
    ((t_alsa_sample16 *)alsa_snd_buf)[j] = s;
}

// Input: hardware format → t_sample (float)
if (alsa_indev[iodev].a_sampwidth == 4) {  // 32-bit
    *fp2 = (t_sample)((t_alsa_sample32 *)alsa_snd_buf)[j] * (1./INT32_MAX);
}
else {  // 16-bit
    *fp2 = (t_sample)((t_alsa_sample16 *)alsa_snd_buf)[j] * 3.051850e-05;
}
```

**JACK backend** (s_audio_jack.c):
```c
// JACK always uses float32, so conversion is trivial
*soundiop++ = (t_sample)*fp3;  // Identity if t_sample=float
*fp3 = (float)*soundiop++;
```

### 3. Architecture Summary

```
┌─────────────────────────────────────────┐
│  Hardware (ALSA/JACK/CoreAudio)         │
│  Format: S16, S32, F32, F64, etc.       │
└─────────────────┬───────────────────────┘
                  │ Conversion happens here
┌─────────────────▼───────────────────────┐
│  I/O Layer (s_audio_*.c)                │
│  - Detects hardware format              │
│  - Converts to/from t_sample            │
│  - Fills ring buffers                   │
└─────────────────┬───────────────────────┘
                  │ Always t_sample (float/double)
┌─────────────────▼───────────────────────┐
│  DSP Core (d_*.c)                       │
│  - All processing uses t_sample         │
│  - Range: [-1.0, 1.0]                   │
│  - Format-agnostic                      │
└─────────────────────────────────────────┘
```

---

## Recommended Solution for Resonix

### Architecture: Pure Core + I/O Adapter

**Core always uses f32** (internally), **I/O layer converts** (at boundaries).

```
┌─────────────────────────────────────────┐
│  CPAL Audio Backend                     │
│  Generic over S: Sample                 │
└─────────────────┬───────────────────────┘
                  │ Conversion in ring buffer
┌─────────────────▼───────────────────────┐
│  I/O Layer (resonix-audio)              │
│  - Ring buffer stores f32               │
│  - Converts S → f32 on write            │
│  - Converts f32 → S on read             │
└─────────────────┬───────────────────────┘
                  │ Always f32
┌─────────────────▼───────────────────────┐
│  Graph Core (resonix-graph)             │
│  - AudioNode always uses f32            │
│  - No knowledge of hardware formats     │
└─────────────────────────────────────────┘
```

---

## Implementation

### Option 1: Convert at Ring Buffer (Recommended)

**Keep AudioNode pure (always f32), convert in I/O layer.**

```rust
// resonix-audio/src/cpal_impl/cpal_audio_output.rs

pub struct CpalAudioOutput<S: Sample> {
    producer: Producer<f32>,  // ← Always f32 internally
    stream: Stream,
    config: StreamConfig,
    ring_buffer_capacity: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sample + SizedSample + Send + 'static> CpalAudioOutput<S> {
    pub fn from_defaults() -> Self {
        // Ring buffer stores f32 (not S)
        let buffer = HeapRb::<f32>::new(ring_buffer_capacity);
        let (producer, consumer) = buffer.split();

        // Audio callback converts f32 → S
        let mut consumer = Consumer::<f32>(consumer);
        let mut last_sample = 0.0f32;

        let mut next_value = move || {
            match consumer.read() {
                Ok(sample) => {
                    last_sample = sample;
                    S::from_sample(sample)  // ← Conversion happens here
                }
                Err(_) => {
                    // Underrun: return last sample converted
                    S::from_sample(last_sample)
                }
            }
        };

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [S], _| write_data(data, channels, &mut next_value),
            err_fn,
            None,
        ).unwrap();

        stream.play().unwrap();

        Self {
            producer: Producer(producer),
            stream,
            config,
            ring_buffer_capacity,
            _phantom: PhantomData,
        }
    }
}

// SystemAudioOutput trait always uses f32
impl<S: Sample> SystemAudioOutput<f32> for CpalAudioOutput<S> {
    fn write_sample(&mut self, sample: f32) -> Result<(), SystemAudioOutputError> {
        // Write f32 to ring buffer
        self.producer.write(sample).map_err(|_| {
            SystemAudioOutputError::WriteError(Box::new(CpalAudioOutputError::WriteError))
        })?;
        Ok(())
    }

    fn ready_for_sample(&self) -> bool {
        !self.producer.is_full()
    }
}
```

**Key points:**
- Ring buffer stores `f32` (not `S`)
- Audio callback converts `f32 → S` using CPAL's `from_sample()`
- Graph writes `f32` to producer
- No generics in `AudioNode` trait

---

### Option 2: Generic AudioNode (Not Recommended)

**Make AudioNode generic over sample type.**

```rust
pub trait AudioNode<S: Sample>: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError>;
}
```

**Problems:**
- Every node implementation becomes generic
- Can't mix different sample types in same graph
- Complex trait bounds everywhere
- Breaks type erasure (`Box<dyn AudioNode>` becomes `Box<dyn AudioNode<f32>>`)

**Verdict:** Don't do this. Keep AudioNode pure.

---

### Option 3: Dual Ring Buffers (Overkill)

**Convert at both ends: Graph writes f32, I/O reads S.**

```rust
pub struct CpalAudioOutput<S: Sample> {
    // Two ring buffers
    internal_producer: Producer<f32>,   // Graph writes here
    external_consumer: Consumer<S>,     // CPAL reads here

    // Conversion thread
    converter_thread: JoinHandle<()>,
}
```

**Problems:**
- Extra thread complexity
- Double buffering overhead
- No performance benefit (conversion happens anyway)

**Verdict:** Unnecessary. Option 1 is simpler.

---

## CPAL's Sample Trait

CPAL provides conversion utilities:

```rust
pub trait Sample: Copy + Clone {
    /// Convert from any sample type
    fn from_sample<S: Sample>(s: S) -> Self;

    /// Convert to any sample type
    fn to_sample<S: Sample>(self) -> S;

    /// Equilibrium value (silence)
    const EQUILIBRIUM: Self;
}

// Implementations:
impl Sample for f32 {
    const EQUILIBRIUM: f32 = 0.0;
    // ...
}

impl Sample for i16 {
    const EQUILIBRIUM: i16 = 0;
    // ...
}
```

**Conversion examples:**

```rust
// f32 → i16
let f: f32 = 0.5;
let i: i16 = i16::from_sample(f);  // → 16383

// i16 → f32
let i: i16 = 16383;
let f: f32 = f32::from_sample(i);  // → 0.499969482

// f32 → f32 (identity)
let f1: f32 = 0.5;
let f2: f32 = f32::from_sample(f1);  // → 0.5 (no-op)
```

**Scaling factors** (CPAL handles automatically):
- **i16:** ±32767 ↔ ±1.0
- **u16:** 0..65535 ↔ -1.0..1.0
- **f32:** -1.0..1.0 ↔ -1.0..1.0 (identity)
- **f64:** -1.0..1.0 ↔ -1.0..1.0 (precision change)

---

## Comparison: Pure Data vs Resonix

| Aspect | Pure Data | Resonix (Proposed) |
|--------|-----------|-------------------|
| **Internal format** | `t_sample` (float/double) | `f32` always |
| **Compile-time config** | Yes (`PD_FLOATSIZE`) | No (always f32) |
| **Conversion location** | I/O backends | Ring buffer |
| **Supported formats** | S16, S24, S32, F32 | Any `cpal::Sample` |
| **Conversion library** | Manual (hardcoded) | `cpal::Sample` trait |
| **I/O isolation** | Per-backend files | Single `CpalAudioOutput<S>` |

---

## Recommended AudioNode Trait (Final)

```rust
// crates/resonix-graph/src/traits/audio_node.rs

pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    /// Process audio with pre-allocated f32 buffers
    ///
    /// # Arguments
    /// * `inputs` - Input audio buffers (range: -1.0 to 1.0)
    /// * `outputs` - Output audio buffers (write range: -1.0 to 1.0)
    /// * `block_size` - Number of samples to process
    ///
    /// # Notes
    /// - Always uses f32 internally (hardware conversion happens in I/O layer)
    /// - Audio range is normalized: [-1.0, 1.0]
    /// - Buffers are pre-allocated and SIMD-aligned
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        block_size: usize,
    ) -> Result<(), AudioNodeRunError>;
}
```

**Why f32 not f64?**
- JACK always uses f32
- Web Audio API uses f32
- CPAL defaults to f32
- f64 offers no practical benefit for audio DSP
- f32 enables better SIMD (8x f32 vs 4x f64 in AVX)

---

## Implementation Checklist

### Phase 1: Update AudioNode Trait
- [ ] Change signature to `&[&[f32]]` and `&mut [&mut [f32]]`
- [ ] Add `block_size: usize` parameter
- [ ] Update 3 existing nodes (ConstantNode, MultiplyNode, OutputNode)

### Phase 2: Update I/O Layer
- [ ] Change ring buffer to store `f32` (not generic `S`)
- [ ] Add conversion in audio callback: `S::from_sample(f32_value)`
- [ ] Update `SystemAudioOutput` trait to always use `f32`
- [ ] Add error handling for underruns (don't unwrap!)

### Phase 3: Test Conversions
- [ ] Test with CPAL's default format (usually f32)
- [ ] Force i16 format and verify conversion works
- [ ] Measure conversion overhead (should be negligible)
- [ ] Verify no clipping or distortion

---

## Performance Considerations

### Conversion Overhead

**Per-sample conversion cost:**
- f32 → f32: ~0 cycles (identity)
- f32 → i16: ~5 cycles (multiply + cast)
- i16 → f32: ~5 cycles (cast + multiply)

**For 48kHz stereo:**
- 96,000 samples/sec
- ~480,000 cycles/sec for conversion
- On 3GHz CPU: 0.016% CPU usage

**Verdict:** Conversion overhead is negligible compared to DSP processing.

### SIMD Benefits of f32

```rust
use std::simd::f32x8;

// Process 8 samples at once (AVX)
for i in 0..block_size/8 {
    let a = f32x8::from_slice(&in1[i*8..]);
    let b = f32x8::from_slice(&in2[i*8..]);
    let result = a * b;
    result.copy_to_slice(&mut out[i*8..]);
}
```

**f64 equivalent** only processes 4 samples at once (AVX), half the throughput.

---

## Migration Example

### Before (Generic, Current)

```rust
pub struct CpalAudioOutput<S: Sample> {
    producer: Producer<S>,  // Generic
    stream: Stream,
}

impl<S: Sample> SystemAudioOutput<S> for CpalAudioOutput<S> {
    fn write_sample(&mut self, sample: S) -> Result<(), _> {
        self.producer.write(sample)?;
        Ok(())
    }
}
```

### After (f32 Internal, Conversion at Boundary)

```rust
pub struct CpalAudioOutput<S: Sample> {
    producer: Producer<f32>,  // Always f32
    stream: Stream,
    _phantom: PhantomData<S>,
}

impl<S: Sample + SizedSample + Send + 'static> CpalAudioOutput<S> {
    pub fn from_defaults() -> Self {
        let buffer = HeapRb::<f32>::new(capacity);
        let (producer, consumer) = buffer.split();

        let mut consumer = Consumer::<f32>(consumer);
        let mut last_sample = 0.0f32;

        let mut next_value = move || {
            consumer.read()
                .unwrap_or(last_sample)
                .tap(|&s| last_sample = s)
                .then(|s| S::from_sample(s))  // Convert f32 → S
        };

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [S], _| {
                for frame in data.chunks_mut(channels) {
                    for sample in frame {
                        *sample = next_value();
                    }
                }
            },
            err_fn,
            None,
        ).unwrap();

        Self { producer: Producer(producer), stream, _phantom: PhantomData }
    }
}

// Trait always uses f32 (regardless of hardware format S)
impl<S: Sample> SystemAudioOutput<f32> for CpalAudioOutput<S> {
    fn write_sample(&mut self, sample: f32) -> Result<(), _> {
        self.producer.write(sample)?;
        Ok(())
    }
}
```

---

## Summary

**Answer: Yes, it's possible and straightforward.**

**Recommendation:**
1. **Keep AudioNode pure:** Always use `f32` slices
2. **Convert at I/O boundary:** Ring buffer stores `f32`, CPAL callback converts to hardware format
3. **Use CPAL's Sample trait:** Handles all conversions automatically
4. **Follow Pure Data's architecture:** Internal format (f32) separate from hardware format (S)

**Benefits:**
- ✅ AudioNode implementations stay simple (no generics)
- ✅ Core is hardware-agnostic
- ✅ CPAL handles scaling factors automatically
- ✅ Negligible conversion overhead (~0.016% CPU)
- ✅ Enables SIMD optimizations (f32x8)

**No downsides:**
- Conversion happens anyway (even Pure Data does it)
- f32 range [-1.0, 1.0] is the audio industry standard
- Hardware ultimately determines format, not our code

---

## References

- Pure Data analysis: [pd-analysis/](./pd-analysis/)
- CPAL Sample trait: [docs.rs/cpal](https://docs.rs/cpal)
- Current code: `crates/resonix-audio/src/cpal_impl/cpal_audio_output.rs`
- AudioNode trait: `crates/resonix-graph/src/traits/audio_node.rs`
