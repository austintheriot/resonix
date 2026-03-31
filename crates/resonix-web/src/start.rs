use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode, so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // enables using info!() macros
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Running startup function from wasm");

    Ok(())
}
