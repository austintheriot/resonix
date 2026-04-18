use std::path::PathBuf;

fn main() {
    // Locate the workspace root by walking up from CARGO_MANIFEST_DIR.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

    let wasm_src = workspace_root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("resonix_wasm_audio_node.wasm");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let wasm_dst = out_dir.join("resonix_wasm_audio_node.wasm");

    if wasm_src.exists() {
        std::fs::copy(&wasm_src, &wasm_dst).expect("failed to copy wasm test fixture");
    } else {
        // Create an empty file so include_bytes! compiles even before the WASM is built.
        // Tests that require real WASM will be gated with #[cfg_attr(not(miri), test)].
        std::fs::write(&wasm_dst, b"").expect("failed to write placeholder wasm");
        println!("cargo:warning=resonix_wasm_audio_node.wasm not found at {wasm_src:?}; run `cargo build -p resonix-wasm-audio-node --target wasm32-unknown-unknown --release` first");
    }

    // Re-run if the wasm binary changes.
    println!("cargo:rerun-if-changed={}", wasm_src.display());
}
