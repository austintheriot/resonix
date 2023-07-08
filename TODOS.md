# Todos

## Audio Graph

- Convert current demos into audio_graph tests

  - Basic audio graph with DAC
  - Sending processor updates via channels (add_node, connect)
  - Sending node updates via channels (set_frequency)
  - Updating audio_context before initializing DAC vs after initializing DAC

- refactor message passing with audio context into a utility function

- Add parallelism with Rayon

- Upmix / downmix DAC audio data instead of ignoring channel and audio output mismatches when moving data from within DAC nodes to actual audio output buffer

- Use snapshot testing for complex audio

- Should DAC be initialized by default always when `"dac"` feature is enabled?

- The return value of a node getting attached to the audio context is a node_index and message sender for sending messages to the audio context for after the process has moved into the audio thread

- BUG: currently, if a NodeHandle tries to update a Node property, the async function will hang indefinitely, because there is no DAC running to run the update. Consider when the `audio_context` should run updates to its nodes IF THERE IS NO DAC - since currently the only time those updates are made are when the DAC is running.

- implement more ergonomic API for creating nodes, adding them to the audio_context, etc.

- allow removing audio nodes

- make sure there are no race conditions when adding multiple nodes simultaneously

  - Make all calls to the audio graph synchronous if DAC has not been initialized yet

- allow multichannel audio connections

- allow microphone input (ADCNode?, AudioInputNode? MicNode?)

- create Buffer player node

- Enable cyclical graphs:
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
