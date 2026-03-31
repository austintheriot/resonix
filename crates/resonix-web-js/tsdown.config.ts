import { defineConfig, Rolldown } from "tsdown";
import fs from "node:fs";
import path from "path";
import { createRequire } from "module";

/**
 * Makes a direct copy of the resonix wasm file to the bundle,
 * to enable callers to instantiate the wasm binary by hand.
 */
const copyResonixWasmFileToBundle = (): Rolldown.Plugin => {
  return {
    name: "copy-wasm-from-dep",
    buildStart() {
      const require = createRequire(import.meta.url);
      // works even with yarn workspace symlinks
      const packageEntry = require.resolve("resonix");
      const packageDirectory = path.dirname(packageEntry);
      const wasmPath = path.join(packageDirectory, "resonix_bg.wasm");
      const wasmBinary = fs.readFileSync(wasmPath);

      this.emitFile({
        type: "asset",
        fileName: "resonix.wasm",
        source: wasmBinary,
      });
    },
  };
};

export default defineConfig({
  dts: true,
  exports: true,
  // preserve worklet script in exports for consumers to provide
  // during audio node instantiation
  // TODO: eventually, if/when `tsdown` supports `?url` loaders,
  // just fetch it directly
  // TODO: could try using Worker loaders as well or inlining script
  entry: ["src/index.ts", "src/resonixProcessor.ts"],
  plugins: [copyResonixWasmFileToBundle()],
});
