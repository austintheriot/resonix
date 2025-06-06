use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
#[allow(dead_code)]
fn it_passes() {
    assert!(Some(123).is_some());
}
