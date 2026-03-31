import * as Resonix from "resonix";
import { RESONIX_PROCESSOR_NAME } from "./common.js";

class ResonixProcessor extends AudioWorkletProcessor {
  private _jsRetainedGraph = Resonix.JsRetainedGraph.new();

  constructor() {
    super();

    this._jsRetainedGraph.print_external_buffer_mappings();
  }

  process(
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
