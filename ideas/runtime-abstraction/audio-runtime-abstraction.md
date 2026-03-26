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

## Option A: `ChannelRuntime` Trait with GATs

Define a trait that abstracts over "how to create a sample channel":

```rust
pub trait ChannelRuntime: 'static {
    type Producer<S: Send + 'static>: Producer<S> + Send + 'static;
    type Consumer<S: Send + 'static>: Consumer<S> + Send + 'static;

    fn channel<S: Send + 'static>(capacity: usize)
        -> (Self::Producer<S>, Self::Consumer<S>);
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

`CpalAudioOutput` takes a `ChannelRuntime` type parameter:

```rust
pub struct CpalAudioOutput<R: ChannelRuntime, S: Sample + Send + 'static> {
    producer: R::Producer<S>,
    stream: Stream,
    config: StreamConfig,
    _phantom: PhantomData<S>,
}

impl<R: ChannelRuntime, S: Sample + ...> CpalAudioOutput<R, S> {
    pub fn from_defaults() -> Result<Self, ...> {
        let (producer, consumer) = R::channel(DEFAULT_CAPACITY);
        // consumer captured in cpal callback...
    }
}
```

Call sites:

```rust
// native
let output = CpalAudioOutput::<RingbufRuntime, f32>::from_defaults()?;

// WASM
let output = WebAudioOutput::<WasmChannelRuntime, f32>::from_defaults()?;

// test
let output = MockAudioOutput::<VecDequeRuntime, f32>::default();
```

**Pros:** Zero-cost, compiler enforces correctness, clean call sites once the runtime type is named.

**Cons:** GAT bounds are verbose and can cause `where` clause explosion as the type parameter propagates up. GAT object safety is limited — can't use `dyn ChannelRuntime` for dynamic dispatch.

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

Combine both: externalize channel construction as the canonical low-level form, and provide `ChannelRuntime` as an optional convenience factory on top.

```rust
// Low-level: explicit channel injection (always works)
let output = CpalAudioOutput::new(my_producer, my_consumer, &device, &config)?;

// High-level: runtime factory (optional convenience)
let output = CpalAudioOutput::with_runtime::<RingbufRuntime>(&device, &config)?;
```

The `ChannelRuntime` trait is **additive** — the `new(P, C, ...)` constructors remain the canonical API. `with_runtime` is sugar for callers who want a one-liner default and don't need to customize the channel.

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

2. **Add Option A on top** when a second runtime is actually needed (e.g., a WASM/WebAudio backend). The `ChannelRuntime` GAT trait makes the most sense once there are at least two concrete implementations to justify the abstraction.

The webrtc.rs article's approach applies directly here — their `Runtime` trait abstracts over async spawn/timers/sockets; the analog for `resonix-audio` is a `ChannelRuntime` trait that abstracts over how samples move between the audio callback context and user code.
