import * as Resonix from "resonix";
import { RESONIX_PROCESSOR_NAME } from "./common.js";

export async function createResonixProcessorFromUrl(
  audioContext: AudioContext,
  url: string,
): Promise<AudioWorkletNode> {
  await audioContext.resume();

  console.log("resonix-web-js: createResonixProcessorFromUrl", {
    url,
    audioContext,
  });
  await audioContext.audioWorklet.addModule(url);

  return new AudioWorkletNode(audioContext, RESONIX_PROCESSOR_NAME);
}

export function printExports() {
  console.log("resonix-web-js: printing raw Rust exports from `resonix`", {
    Resonix,
  });
}

export * from "resonix";
