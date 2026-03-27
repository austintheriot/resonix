import "./style.css";
import { default as init, Graph } from "resonix/resonix.js";

async function main() {
  console.log("Running `main`");
  await init();

  const graph = Graph.test_with_block_size(1);
  console.log("Graph initialized from JavaScript", { graph });
}

main();
