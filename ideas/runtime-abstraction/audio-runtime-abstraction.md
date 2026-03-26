# Audio Runtime Abstraction

## Motivation

A lot of audio work is highly platform-dependent:

- `cpal` works great natively but has limited WASM/web support
- `ringbuf` uses threading and `unsafe` in ways that may not work in single-threaded WASM contexts
- Some channel implementations work on web; others don't
- Mocking is harder than it should be

The goal is to abstract over the I/O and runtime primitives used by `resonix-audio` so that callers can choose the right implementation for their platform, or swap in their own.

**Reference**: [webrtc.rs: Building Async-Friendly WebRTC](https://webrtc.rs/blog/2026/01/31/async-friendly-webrtc-architecture.html) — same problem applied to async runtimes and UDP sockets.

---

## The Core Problem

`CpalAudioOutput` and `CpalAudioInput` are already generic over `P: Producer<S>` / `C: Consumer<S>` at the type level, but their constructors **create the `ringbuf` channel pair internally**. The caller can never inject a different implementation.

```rust
// from_defaults() does this internally — hidden from caller
let rb = HeapRb::<S>::new(capacity);
let (prod, cons) = rb.split();
// cons is captured in the cpal callback closure
// prod lives in self
```

The mock `producer()` / `consumer()` methods on `SystemAudioInput` / `SystemAudioOutput` returning `Option<Box<dyn ...>>` are a symptom of the same issue — the trait is working around the fact that the channel is hidden inside the implementation.

---

## Option A: `ChannelRuntime` Trait with Dynamic Dispatch

Mirroring the webrtc.rs approach directly: a `Runtime` trait whose methods return boxed trait objects. The article's `wrap_udp_socket` returns `Box<dyn AsyncUdpSocket>`; the audio analog is `create_channel` returning `(Box<dyn Producer<S>>, Box<dyn Consumer<S>>)`.

```rust
pub trait ChannelRuntime: Send + Sync + 'static {
    fn create_channel<S: Send + 'static>(
        &self,
        capacity: usize,
    ) -> (Box<dyn Producer<S>>, Box<dyn Consumer<S>>);
}
```

Provided implementations:

```rust
pub struct RingbufRuntime;       // native default (lock-free, cache-friendly)
impl ChannelRuntime for RingbufRuntime { ... }

pub struct VecDequeRuntime;      // single-threaded / WASM-safe
impl ChannelRuntime for VecDequeRuntime { ... }

pub struct StdMpscRuntime;       // std::sync::mpsc
impl ChannelRuntime for StdMpscRuntime { ... }
```

`CpalAudioOutput` stores the boxed producer from the runtime:

```rust
pub struct CpalAudioOutput<S: Sample + Send + 'static> {
    producer: Box<dyn Producer<S>>,
    stream: Stream,
    config: StreamConfig,
}

impl<S: Sample + ...> CpalAudioOutput<S> {
    pub fn from_runtime(
        runtime: &dyn ChannelRuntime,
        device: &Device,
        config: &StreamConfig,
    ) -> Result<Self, ...> {
        let (producer, consumer) = runtime.create_channel(DEFAULT_CAPACITY);
        // consumer captured in cpal callback...
    }
}
```

Call sites:

```rust
// native
let output = CpalAudioOutput::from_runtime(&RingbufRuntime, &device, &config)?;

// WASM
let output = WebAudioOutput::from_runtime(&WasmChannelRuntime, &device, &config)?;

// test
let output = CpalAudioOutput::from_runtime(&VecDequeRuntime, &device, &config)?;
```

**Pros:** No GATs, no type parameter propagation, supports `dyn ChannelRuntime` for dynamic dispatch. The boxing cost occurs once at construction — not per sample — so it's negligible.

**Cons:** Sample reads/writes go through `Box<dyn Producer<S>>` / `Box<dyn Consumer<S>>`, adding a vtable indirection on every call. Whether this matters depends on whether `Producer`/`Consumer` calls are in the hot path.

---

## Option B: Externalize Channel Construction

No new trait — just stop hiding channel creation inside constructors. Add constructors that accept the channel pair directly:

```rust
impl<S: Sample + Send, P: Producer<S> + Send, C: Consumer<S> + Send + 'static>
    CpalAudioOutput<S, P>
{
    pub fn new(
        producer: P,
        consumer: C,
        device: &Device,
        config: &StreamConfig,
    ) -> Result<Self, CpalAudioOutputError> { ... }
}
```

Call sites:

```rust
// native — caller creates ringbuf
let rb = HeapRb::<f32>::new(1024);
let (prod, cons) = rb.split();
let output = CpalAudioOutput::new(
    RingbufProducer(prod), RingbufConsumer(cons), &device, &config
)?;

// test — caller creates VecDeque-backed channel and holds both ends
let (prod, cons) = vec_deque_channel::<f32>(1024);
let output = MockAudioOutput::new(prod);
// cons is held by the test — no need for a special producer()/consumer() method
```

**Pros:** Very simple, no new abstractions, immediately solves the mock awkwardness.

**Cons:** Slightly more verbose at call sites. No built-in "give me the right channel for this platform" default.

---

## Option C: Hybrid (recommended)

Combine both: externalize channel construction as the canonical low-level form (Option B), and provide `ChannelRuntime` as an optional convenience factory on top (Option A).

```rust
// Low-level: explicit channel injection (always works, zero-cost)
let output = CpalAudioOutput::new(my_producer, my_consumer, &device, &config)?;

// High-level: runtime factory (optional convenience, boxes at construction only)
let output = CpalAudioOutput::from_runtime(&RingbufRuntime, &device, &config)?;
```

The `ChannelRuntime` trait is **additive** — the `new(P, C, ...)` constructors remain the canonical API and keep the concrete types. `from_runtime` is sugar for callers who want a one-liner default and are fine with `Box<dyn Producer<S>>` internally.

---

## The Mock Trait Issue (orthogonal but important)

The `producer()` / `consumer()` methods on `SystemAudioInput` / `SystemAudioOutput` returning `Option<Box<dyn ...>>` are a design smell. They exist to work around the channel being hidden. Once the channel is externalized:

- Remove `producer()` / `consumer()` from the traits entirely
- `MockAudioInput` / `MockAudioOutput` expose their handles as direct methods on the concrete types
- Test code either holds the other end of the channel from construction, or casts to the concrete mock type

A `SystemAudioOutput` shouldn't need to know it's a mock. The trait should only describe observable audio behavior.

---

## Recommended Approach

1. **Start with Option B** — remove `from_defaults()` constructors that hide channels, add `new(P, C, ...)` constructors. Clean up `producer()` / `consumer()` from the traits. This is the 80% solution with minimal new abstraction.

2. **Add Option A on top** when a second runtime is actually needed (e.g., a WASM/WebAudio backend). The `ChannelRuntime` trait makes the most sense once there are at least two concrete implementations to justify the abstraction.

The webrtc.rs article's approach applies directly here — their `Runtime` trait abstracts over async spawn/timers/sockets; the analog for `resonix-audio` is a `ChannelRuntime` trait that abstracts over how samples move between the audio callback context and user code.
