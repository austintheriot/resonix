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

// TODO: move logic of `ResonixProcessor` into a
// `ResonixProcessorCore` struct for easier/env-agnostic testing,
// then just call it into it from here
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
    inputs: Float32Array[][],
    outputs: Float32Array[][],
    _parameters: Record<string, Float32Array>,
  ): boolean {
    const frequency = this._frequency;
    if (frequency === undefined) {
      return true;
    }

    if (!this._jsRetainedGraph) {
      return true;
    }

    return this._jsRetainedGraph.process(
      inputs,
      outputs,
      currentTime,
      sampleRate,
    );
  }
}

console.log("From ResonixProcessor worklet script: registering processor", {
  RESONIX_PROCESSOR_NAME,
  ResonixProcessor,
});
registerProcessor(RESONIX_PROCESSOR_NAME, ResonixProcessor);
