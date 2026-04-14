import "./style.css";
import { ResonixNode } from "resonix-web-js";
import processorUrl from "resonix-web-js/worklet/ResonixProcessor.worklet?url";
import wasmUrl from "resonix-web-js/resonix.wasm?url";

async function main() {
  console.log("web-frontend-vite: running `main`");

  // web audio must be started from a user interaction
  const button = document.createElement("button");
  button.textContent = "Start audio";

  let started = false;
  let audioContext: AudioContext | undefined;
  button.onclick = async () => {
    if (!audioContext) {
      audioContext = new AudioContext();
      const wasmInitSource = fetch(wasmUrl).then((res) => res.bytes());
      const resonixNode = await ResonixNode.newWithInit({
        audioContext,
        wasmInitSource,
        processorUrl,
        frequency: 440,
      });
      resonixNode.connect(audioContext.destination);
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
