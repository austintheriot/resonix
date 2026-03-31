import * as Resonix from "resonix";

export function printExports() {
  console.log("resonix-web-js: printing raw Rust exports from `resonix`", { Resonix });
}

export * from "resonix";
