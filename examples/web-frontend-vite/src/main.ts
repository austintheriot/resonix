import "./style.css";
import { printExports, createResonixProcessorFromUrl } from "resonix-web-js";
import resonixProcessorUrl from "resonix-web-js/ResonixProcessor?url";
import resonixWasmUrl from "resonix-web-js/resonix.wasm?url";

async function main() {
  console.log("web-frontend-vite: running `main`");
  printExports();

  // web audio must be started from a user interaction
  const button = document.createElement("button");
  button.textContent = "Start audio";

  let started = false;
  let audioContext: AudioContext | undefined;
  button.onclick = async () => {
    if (!audioContext) {
      const wasmBytes = await fetch(resonixWasmUrl).then((res) => res.bytes());
      const wasmArrayBuffer = wasmBytes.buffer;
      audioContext = new AudioContext();
      const resonixProcessor = await createResonixProcessorFromUrl(
        audioContext,
        wasmArrayBuffer,
        resonixProcessorUrl,
        440,
      );
      resonixProcessor.connect(audioContext.destination);
    }

    if (started) {
      started = false;
      button.textContent = "Start audio";
      audioContext.suspend();
    } else {
      started = true;
      button.textContent = "Stop audio";
      audioContext.resume();
    }
  };

  document.body.appendChild(button);
}

main();
