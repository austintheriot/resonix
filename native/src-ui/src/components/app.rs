use js_sys::Promise;
use log::info;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use yew::{function_component, html, prelude::*};

#[wasm_bindgen(module = "@tauri-apps/api/event")]
extern "C" {
    #[wasm_bindgen(js_name = emit)]
    fn emit(event: String);

    #[wasm_bindgen(js_name = emit)]
    fn emit_with_payload(event: String, arguments: JsValue);
}

#[wasm_bindgen(module = "@tauri-apps/api/tauri")]
extern "C" {
    #[wasm_bindgen(js_name = invoke)]
    fn invoke(event: String) -> Promise;
}

#[function_component(App)]
pub fn app() -> Html {
    use_effect_with_deps(
        move |_| {
            info!("emitting event from the front-end");
            emit(String::from("event-name"));
            info!("invoking increment functin ofrom front-end");
            invoke(String::from("increment"));
            || {}
        },
        (),
    );

    html! {
        <div>
            <button
                onclick={|_| emit(String::from("event-name"))}
            >
                {"Emit event"}
            </button>
            <button
                onclick={|_| {
                    wasm_bindgen_futures::spawn_local(async {
                        let new_count_js_value = JsFuture::from(invoke(String::from("increment"))).await.unwrap();
                        let new_count = JsValue::as_f64(&new_count_js_value).unwrap();
                        info!("{:?}", new_count);
                    })
                }}
            >
                {"Invoke command"}
            </button>
        </div>
    }
}
