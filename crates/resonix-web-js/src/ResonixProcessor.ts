// Wasm assumes a `TextEncoder` / `TextDecoder` implementation,
// but none is available in the `AudioWorkletGlobalScope`
import "./polyfillTextEncoder.js";
import init, { JsRetainedGraph } from "resonix";
import {
  RESONIX_PROCESSOR_NAME,
  type ResonixInitMessage,
  type ResonixProcesorIncomingMessage,
  type ResonixProcesorOutgoingMessage,
} from "./common.js";

class ResonixProcessor
  extends AudioWorkletProcessor
  implements EventListenerObject
{
  private _jsRetainedGraph: JsRetainedGraph | undefined;
  private _frequency: number | undefined;

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
  private async _init(initMessage: ResonixInitMessage): Promise<void> {
    await init(initMessage.wasmInitSource);
    this._jsRetainedGraph = JsRetainedGraph.new();
    this._jsRetainedGraph.print_external_buffer_mappings();
    this._frequency = initMessage.frequency;
    this._postMessage({ tag: "ready" });
  }

  private _postMessage(
    message: ResonixProcesorOutgoingMessage,
    transfer: Transferable[] = [],
  ): void {
    this.port.postMessage(message, transfer);
  }

  // TODO: strongly type these messages
  public onPortMessage(event: MessageEvent<ResonixProcesorIncomingMessage>) {
    console.log("###### ResonixProcessor.onmessage", { event });
    switch (event.data.tag) {
      case "init":
        this._init(event.data);
        break;
      default:
        console.error(
          "Unexpected case reached in Resonix.Processor.onPortMessage",
        );
    }
  }

  public process(
    _inputs: Float32Array[][],
    outputs: Float32Array[][],
    _parameters: Record<string, Float32Array>,
  ): boolean {
    const frequency = this._frequency;
    if (frequency === undefined) {
      return true;
    }

    // just test getting sound going
    outputs.forEach((output) => {
      output.forEach((channel) => {
        channel.forEach((_sample, sampleIndex) => {
          const sampleTime = currentTime + sampleIndex / sampleRate;
          channel[sampleIndex] = Math.sin(2 * Math.PI * frequency * sampleTime);
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
