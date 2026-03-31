import { defineConfig } from "tsdown";

export default defineConfig({
  dts: true,
  exports: true,
  // preserve worklet script in exports for consumers to provide
  // during audio node instantiation
  // TODO: eventually, if/when `tsdown` supports `?url` loaders,
  // just fetch it directly
  // TODO: could try using Worker loaders as well or inlining script
  entry: ["src/index.ts", "src/resonixProcessor.ts"],
});
