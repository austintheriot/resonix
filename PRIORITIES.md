# Priorities & Distinguishing Features for Resonix

### Portability / Embeddability

- Highly embeddable/portable
  - Actual graph DSP algorithms make very few assumptions about the calling env (except for the presence of an allocator, though that could even be addressed, if desired)
  - Embeddable to most native envs (via Rust)
  - Runnable on the web (via Wasm)
  - Runnable on Desktop/iOS/Android devices via Rust
  - Support systems that are currently unsupported by Max like Linux

### Writing/patching & running directly in the browser

- Provide env-agnostic API bindings to allow supporting multiple GUI frontends? This would enable both low-level rendering on small devices and more bells-and-whistles rendering on high-power devices (that is, desktop native or browser-based rendering)

### Modular

- Multi-tiered library, available and usable at any level:

  - Low-level core logic with no-std: can be run in any env - can called manually from the outside to progress audio loops/timers/etc. -- works for both real-time audio as well as for offline audio processing/rendering
  - Medium-level: Rust wraps utilities for bare-metal stuff that interacts with I/O, includes resource management, etc.
  - High-level visual scripting environment that surfaces the low-level library as a GUI (like Max/Pure Data) that can be used by non-programmers for high-level audio programming (but still provides users the ability to "dip" into the low-level stuff as much as they want for maximum control/performance)

- The Rust code exposes internal Traits and accepts generic interfaces where possible/practical, which can be implemented by other users to swap core logic

### Language Agnostic

- Built with Rust bindings in mind, with plans to expend to other languages (such as JavaScript / Typescript, Python, etc.)

### Memory Safe & Reliable

- Bugs and crashes abound in other software
- Resonix is memory safe by virtue of using Rust
- High degree of testing in place

## Weak Spots in Max (and Pure Data?)

- Debugability
- No classical music primitives
- Not strong typing at all (int, float, list, symbol, and bang)
- Differentiation between message/event timing and audio timing (causes weird bugs to happen)

## Possible Directions to Go in the Future

- Allow debug/time-travel mode for traveling frame-by-frame through a patch
- Build classical-music primitives into the language & environment
- More strongly typed (less implicit type coercion), more static guarantees
- Multiplayer patching via CRDT
- Integrate visual creative coding from the beginning? (via WegGPU?)
  - Akin to Max Jitter
- Compile-time audio-graph compilation
  - Maximum performance
  - Super fast start-up time (just initializing pre-compiled data structures directly into memory)
  - Zero dynamic allocation
- Convenient, macro-based scripting language for Rust

## Use cases for Resonix

- Compiling Rust code directly for CLI creative coding.
- Use as a plugin from within bevy for games/algorithmic audio
- Making a patching/visual programming GUI akin to Max / Pure Data
- Enabling projects like realtime collaborative music applications (iOS/Android)
- Enable live, distributed music-making via smart phone controllers
- 3D game environment, where users interact with 3D patch objects for collaborative music-making
- Run as a web-based program with UI elements: dragging cursor around a box for granular/concatenative synthesis.
- Compiling for JS/WASM and embedding musical elements in a Three.js project

## Testing ideas

- ✔️ General unit & integration tests
- ✔️ Running Miri for testing soundness
- Snapshot testing (for audio outputs)
- Fuzz testing
- See cpal testing practices for testing audio output on native code

## Other Rust Library Inspirations

- cpal (cross-platform, automated testing, CI)
- nannou (creative coding, wraps cpal)
- bevy (highly complex, ergonomic, multiplatform)
- tauri (multiplatform)
