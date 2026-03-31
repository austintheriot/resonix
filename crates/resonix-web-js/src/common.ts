export const RESONIX_PROCESSOR_NAME = "resonix-processor";

export type ResonixInitMessage = {
  tag: "init";
  wasmBytes: ArrayBuffer;
  frequency: number;
};

export type ResonixReadyMessage = {
  tag: "ready";
};

export type ResonixNodeOutgoingMessage = ResonixInitMessage;
export type ResonixNodeIncomingMessage = ResonixReadyMessage;

// re-export from ResonixProcessor's perspective
export type ResonixProcesorIncomingMessage = ResonixNodeOutgoingMessage;
export type ResonixProcesorOutgoingMessage = ResonixNodeIncomingMessage;
