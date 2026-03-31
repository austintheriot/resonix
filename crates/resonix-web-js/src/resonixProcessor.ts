// Wasm assumes a `TextEncoder` / `TextDecoder` implementation,
// but none is available in the `AudioWorkletGlobalScope`
import "./polyfillTextEncoder.js";
import init, { JsRetainedGraph } from "resonix";
import { RESONIX_PROCESSOR_NAME } from "./common.js";

class ResonixProcessor extends AudioWorkletProcessor {
  private _jsRetainedGraph: JsRetainedGraph | undefined;

  constructor() {
    super();

    // TODO: register the class itself as an EventListenerObject
    this.port.onmessage = (event) => this.onmessage(event.data);
    this.port.onmessageerror = (event) => this.onmessageerror(event.data);
  }

  public onmessageerror(event: MessageEvent): void {
    console.log("###### ResonixProcessor.onmessageerror", { event });
  }

  public onmessage(event: MessageEvent<{ wasmBytes: ArrayBuffer }>) {
    console.log("###### ResonixProcessor.onmessage", { event });
    if (event.type === "send-wasm-module") {
      init(WebAssembly.compile(event.data.wasmBytes)).then(() => {
        this.port.postMessage({ type: "wasm-module-loaded" });
      });

      // TODO:
      this._jsRetainedGraph = JsRetainedGraph.new();
      this._jsRetainedGraph.print_external_buffer_mappings();
    }
  }

  public process(
    _inputs: Float32Array[][],
    outputs: Float32Array[][],
    _parameters: Record<string, Float32Array>,
  ): boolean {
    // just test getting sound going
    const FREQUENCY = 440.0;
    outputs.forEach((output) => {
      output.forEach((channel) => {
        channel.forEach((_sample, sampleIndex) => {
          const sampleTime = currentTime + sampleIndex / sampleRate;
          channel[sampleIndex] = Math.sin(2 * Math.PI * FREQUENCY * sampleTime);
        });
      });
    });

    return true;
  }
}

console.log("From ResonixProcessor worklet script: registering processor", {
  RESONIX_PROCESSOR_NAME,
  ResonixProcessor,
});
registerProcessor(RESONIX_PROCESSOR_NAME, ResonixProcessor);
