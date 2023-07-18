# Todos

## Audio Graph

- allow microphone input (ADCNode?, AudioInputNode? MicNode?)
  - Consider how multiple audio inputs should interact with the rest of the AudioContext organization
    - One option here is that the Processor itself would be wrapped in an `Arc<Mutex<T>>` (locked for the entire synchronous loop of an audio out rendering loop), but *internally*, no Nodes or Connections would be locked behind a Mutex. This would allow input and output loops to access the processor, but without slowing down the main render loop too much


- Make typing on NodeHandle messages stronger
- 
- Add parallelism with Rayon (or just plain threads?)

- Use an actual newtype for NodeUid & ConnectionUid (same one for both? ContextUid)

- speed up computation by enabling multichannel data to be stored in an array instead of a vec?

- Upmix / downmix DAC audio data instead of ignoring channel and audio output mismatches when moving data from within DAC nodes to actual audio output buffer
  - Use logical channels and map from logical to hardware channels. See [Max docs](https://docs.cycling74.com/max8/tutorials/04_mspaudioio) for how they do it
  - Allow pre-configuring audio sources and outputs (via CLI?)

- Allow serializing/deserializing AudioContext from memory for most efficient audio initialization

- Use snapshot testing for complex audio

- Should DAC be initialized by default always when `"dac"` feature is enabled?

- implement more ergonomic API for creating nodes, adding them to the audio_context, etc.
  - See HashMap's `Entry` for ideas here
  - Each action could return an owned value that carries a reference back to the `&mut AudioContext`, so that infinite chaining methods would be possible

- allow removing audio nodes

- Optimization idea:
  - Once a visit_order is created, actually arrange the nodes in memory that way (using a Vec) for maximizing cache hits

- make sure there are no race conditions when adding multiple nodes asynchronously at the same time

- create Buffer player node

- Enable cyclical graphs (? Is this possible ? Especially with any degree of parallelism ?):
  - One path forward for supporting cyclical graphs
    - During the phase where the visit_order is being constructed, mark any nodes that were moved to the end of the array. If those nodes are visited again, we can assume that they require cyclical references, and just add them as-is to the processing order. With this logic, on the first run, all incoming connections to cyclical nodes will have data of 0.0 on the first pass but will get data on subsequent passes as their child nodes process data.
    - One sticky part here is figuring out the correct processing order for these nodes: how can we guarantee that long, cyclical chains are ordered correctly? Process them as a sub-tree? See cyclical graph unfolding for a possible solution here: basically only allow one level of recursion on every iteration.

## Granular Synthesizer

- add handle below buffer selection to allow dragging

- scale amplitude by number of ACTIVE grains rather than by number of channels?

- use a `Cached` struct for `selection_start_in_samples` rather than doing that by hand with separate properties / functions

- allow playing through source unaltered (add vertical line / drag-and-drop icon above buffer selection)

- When lowering number of channels, allow old channels to fade out (?)

- disable recording button until something has actually been recorded

- Show hoverable tooltips for icon buttons

- Enable sampling from multi-channel audio input

- split granular synth into its own repo / separate website entry before adding more features?

- Show recording buffer visualization (or at least some sort of indication recording is taking place)

- Interpolate changes in Gain (especially when pausing/playing)

- Name downloaded file something more like: "name_of_audio_granulated.wav" or similar

- Move audio PROCESSING into a Web Worker thread so that `cpal` merely has to request audio data at the appropriate time

  - Keeps main thread / audio processing from getting locked up by UI updates and vice versa
  - Keeps WebAudio context on the main thread (where it has to be)

- Share a single audio context that is initialized (?) at init time?
- Memoize decoded audio from previous files? To prevent stutter on change?
- Enable dragging the current buffer selection window?

- Web

  - Refactor visual representation of current audio buffer:
    - use an svg <path /> element?
    - probably would be best to use a canvas to do this

- CLI

  - Re-sample any audio files that don't match the current sample rate.

- Native

  - Make a Native app using Tauri that relies on current web view with Rust running natively under the hood?
  - Use `serde_wasm_bindgen` instead of message passing between backend and front end to prevent JSON-ifying buffers?

- More audio tools / effects:
  - Recording
  - Reverb
  - Delay

Visual effects: - WebGL: particles that react / correspond to audio grains - Show audio output as a sample window? - Or just show current amplitude output with simple bars

---

General Fixes:

- Correct sample rate for mp3 audio for all environments.
