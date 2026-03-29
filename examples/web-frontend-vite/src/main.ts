import "./style.css";
import { default as init, JsRetainedGraph } from "resonix/resonix.js";

async function main() {
  console.log("Running `main`");
  await init();

  const graph = JsRetainedGraph.new();
  console.log("Graph initialized from JavaScript", { graph });
  graph.print_external_buffer_mappings();
}

main();
