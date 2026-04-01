import {
  RESONIX_PROCESSOR_NAME,
  type ResonixNodeIncomingMessage,
  type ResonixNodeOutgoingMessage,
  type WasmInitSource,
} from "./common.js";

export type MaybePromiseWasmInitSource =
  | WasmInitSource
  | Promise<WasmInitSource>;

export interface ResonixNodeInitOpts {
  /** must only be passed on the first instantiation */
  processorUrl?: string;
  audioContext: AudioContext;
  wasmInitSource: MaybePromiseWasmInitSource;
  frequency: number;
}

export interface ResonixNodeConstructorOpts {
  audioContext: AudioContext;
}

export class ResonixNode extends AudioWorkletNode {
  private static _workletModuleEvaluated = false;
  private _resolvers = Promise.withResolvers<void>();
  private _initStarted = false;

  constructor({ audioContext }: ResonixNodeConstructorOpts) {
    super(audioContext, RESONIX_PROCESSOR_NAME);
  }

  private _postMessage(
    message: ResonixNodeOutgoingMessage,
    transfer: Transferable[] = [],
  ): void {
    this.port.postMessage(message, transfer);
  }

  /**
   * Constructs new node & initializes it, returning the
   * node when initialization is complete.
   */
  public static async newWithInit(
    opts: ResonixNodeConstructorOpts & ResonixNodeInitOpts,
  ): Promise<ResonixNode> {
    await ResonixNode.ensureModuleEvaulated(opts);
    const node = new ResonixNode(opts);
    await node.init(opts);
    return node;
  }

  public static async ensureModuleEvaulated({
    audioContext,
    processorUrl,
  }: {
    audioContext: AudioContext;
    processorUrl?: string;
  }): Promise<void> {
    // TODO: create an async read-write lock to prevent race conditions
    if (!ResonixNode._workletModuleEvaluated) {
      if (!processorUrl) {
        throw new Error(
          "Attempted to init a `ResonixNode` without a `processorUrl`",
        );
      }
      await audioContext.audioWorklet.addModule(processorUrl);
      ResonixNode._workletModuleEvaluated = true;
    }
  }

  /**
   * Begins initialization of the internal wasm modules & audio thread.
   *
   * Will throw:
   * - If you `init` multiple nodes with the same underlying memory
   *   simultaneously (e.g. `Response`). To prevent issues, `.clone()`
   *   any responses before submitting them.
   */
  public async init(opts: ResonixNodeInitOpts): Promise<void> {
    if (this._initStarted) {
      return this._resolvers.promise;
    }

    const [wasmInitSource] = await Promise.all([
      opts.wasmInitSource,
      ResonixNode.ensureModuleEvaulated(opts),
    ]);

    this.port.onmessage = (event) => this.onPortMessage(event);
    this.port.onmessageerror = (event) => this.onPortMessageError(event);
    this._initStarted = true;
    this._postMessage(
      {
        tag: "init",
        wasmInitSource,
        frequency: opts.frequency,
      },
      // DON'T transfer wasm init source here--caller may
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
