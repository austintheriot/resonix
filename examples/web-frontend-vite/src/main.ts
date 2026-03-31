import "./style.css";
import { printExports, createResonixProcessorFromUrl } from "resonix-web-js";
import resonixProcessorUrl from "resonix-web-js/resonixProcessor?url";

async function main() {
  console.log("web-frontend-vite: running `main`");
  printExports();

  // web audio must be started from a user interaction
  const button = document.createElement("button");
  button.textContent = "Start audio";
  button.onclick = async () => {
    const audioContext = new AudioContext();

    const resonixProcessor = await createResonixProcessorFromUrl(
      audioContext,
      resonixProcessorUrl,
    );

    resonixProcessor.connect(audioContext.destination);
  };

  document.body.appendChild(button);
}

main();
