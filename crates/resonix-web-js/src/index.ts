import * as Resonix from "resonix";
import { RESONIX_PROCESSOR_NAME } from "./common.js";
import ResonixNode from "./ResonixNode.js";

export const wasmUrl = new URL("resonix/resonix_bg.wasm", import.meta.url).href;

export async function createResonixProcessorFromUrl(
  audioContext: AudioContext,
  wasmBytes: ArrayBuffer,
  url: string,
  // TODO: just for testing
  frequency: number,
): Promise<ResonixNode> {
  console.log("resonix-web-js: createResonixProcessorFromUrl", {
    url,
    audioContext,
  });
  await audioContext.audioWorklet.addModule(url);
  const resonixNode = new ResonixNode(audioContext, RESONIX_PROCESSOR_NAME);
  await resonixNode.init(wasmBytes, frequency);

  return resonixNode;
}

export function printExports() {
  console.log("resonix-web-js: printing raw Rust exports from `resonix`", {
    Resonix,
  });
}

export * from "resonix";
