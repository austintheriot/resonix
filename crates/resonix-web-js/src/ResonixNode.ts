import type {
  ResonixNodeIncomingMessage,
  ResonixNodeOutgoingMessage,
} from "./common.js";

export default class ResonixNode extends AudioWorkletNode {
  private _resolvers = Promise.withResolvers<void>();
  private _initStarted = false;

  private _postMessage(
    message: ResonixNodeOutgoingMessage,
    transfer: Transferable[] = [],
  ): void {
    this.port.postMessage(message, transfer);
  }

  // TODO: eventually, output constructor result here
  public init(wasmBytes: ArrayBuffer, frequency: number): Promise<void> {
    if (this._initStarted) {
      return this._resolvers.promise;
    }

    this._initStarted = true;

    // TODO: register the class itself as an EventListenerObject
    this.port.onmessage = (event) => this.onPortMessage(event);
    this.port.onmessageerror = (event) => this.onPortMessageError(event);

    this._postMessage(
      {
        tag: "init",
        wasmBytes,
        frequency,
      },
      // DON'T transfer buffer here--caller may
      // want to instantiate more than one Node
      // with the same buffer
      [],
    );

    return this._resolvers.promise;
  }

  // Handle an uncaught exception thrown in the PitchProcessor.
  public onPortMessageError(event: MessageEvent) {
    console.log("###### ResonixNode.onmessageerror", { event });
  }

  public onPortMessage(event: MessageEvent<ResonixNodeIncomingMessage>) {
    console.log("###### ResonixNode.onmessage", { event });

    switch (event.data.tag) {
      case "ready":
        this._resolvers.resolve();
        break;
      default:
        console.error("Unexpected case reached in ResonixNode.onPortMessage");
    }
  }
}
