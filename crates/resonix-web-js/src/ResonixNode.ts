export default class ResonixNode extends AudioWorkletNode {
  private _resolvers = Promise.withResolvers<void>();

  public init(wasmBytes: ArrayBuffer): Promise<void> {
    // TODO: register the class itself as an EventListenerObject
    this.port.onmessage = (event) => this.onmessage(event.data);
    this.port.onmessageerror = (event) => this.onmessageerror(event.data);

    this.port.postMessage(
      {
        type: "send-wasm-module",
        wasmBytes,
      },
      [wasmBytes],
    );

    return this._resolvers.promise;
  }

  // Handle an uncaught exception thrown in the PitchProcessor.
  public onmessageerror(event: MessageEvent) {
    console.log("###### ResonixNode.onmessageerror", { event });
  }

  public onmessage(event: MessageEvent) {
    console.log("###### ResonixNode.onmessage", { event });

    if (event.type === "wasm-module-loaded") {
      // assume this means the module is loaded
      this._resolvers.resolve();
    }
  }
}
