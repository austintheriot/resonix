use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub mod wasm_runner_works {
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn should_pass_constructor_args_to_wasm() {
        assert_eq!(1 + 1, 3)
    }
}
