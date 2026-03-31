// Wasm assumes a `TextEncoder` / `TextDecoder` implementation,
// but none is available in the `AudioWorkletGlobalScope`
import "./polyfillTextEncoder.js";
import init, { JsRetainedGraph } from "resonix";
import { RESONIX_PROCESSOR_NAME } from "./common.js";

class ResonixProcessor
  extends AudioWorkletProcessor
  implements EventListenerObject
{
  private _jsRetainedGraph: JsRetainedGraph | undefined;

  constructor() {
    super();

    this.port.onmessage = (event) => this.onPortMessage(event);
    this.port.onmessageerror = (event) => this.onPortMessageError(event);
  }

  public handleEvent(object: unknown): void {
    console.log("###### ResonixProcessor.handleEvent", { object });
  }

  public onPortMessageError(event: MessageEvent): void {
    console.log("###### ResonixProcessor.onmessageerror", { event });
  }

  // TODO: strongly type these messages
  private async _handleWasmModuleBinary(wasmBytes: ArrayBuffer): Promise<void> {
    const wasmModule = await WebAssembly.compile(wasmBytes);
    await init(wasmModule);
    this._jsRetainedGraph = JsRetainedGraph.new();
    this._jsRetainedGraph.print_external_buffer_mappings();
    this.port.postMessage({ type: "wasm-module-ready" });
  }

  // TODO: strongly type these messages
  public onPortMessage(
    event: MessageEvent<{ type: string; wasmBytes: ArrayBuffer }>,
  ) {
    console.log("###### ResonixProcessor.onmessage", { event });
    if (event.data.type === "send-wasm-module") {
      this._handleWasmModuleBinary(event.data.wasmBytes);
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
