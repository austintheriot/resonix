use js_sys::{Function, Promise};
use log::info;
use native_common::{AppEventName, AppPayload, GlobalEvent};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use wasm_bindgen_futures::JsFuture;
use yew::{function_component, html, prelude::*};

#[wasm_bindgen(module = "@tauri-apps/api/event")]
extern "C" {
    #[wasm_bindgen(js_name = emit)]
    fn emit(event: &str);

    #[wasm_bindgen(js_name = emit)]
    fn emit_with_payload(event: &str, arguments: JsValue);

    /// Returns an unlisten callback Promise(Function)
    #[wasm_bindgen(js_name = listen )]
    fn listen(event: &str, callback: &Function) -> Promise;
}

#[wasm_bindgen(module = "@tauri-apps/api/tauri")]
extern "C" {
    /// Returns whatever value has been defined in the backend
    #[wasm_bindgen(js_name = invoke)]
    fn invoke(event: &str) -> Promise;
}

#[function_component(App)]
pub fn app() -> Html {
    let count_state_handle = use_state(|| 0);

    use_effect_with_deps(
        {
            let count_state_handle = count_state_handle.clone();
            move |_| {
                // get initial count from backend
                {
                    let count_state_handle = count_state_handle.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let initial_count = JsFuture::from(invoke("get_count")).await.unwrap();
                        let initial_count = JsValue::as_f64(&initial_count).unwrap() as u32;
                        count_state_handle.set(initial_count);
                    });
                }

                // listen for counter-changed events
                {
                    let count_state_handle = count_state_handle.clone();
                    let handle_listen = Closure::wrap(Box::new(move |payload: JsValue| {
                        let deserialized_payload: GlobalEvent<Option<AppPayload>> =
                            JsValue::into_serde(&payload).unwrap();
                        info!("deserialized_payload = {:#?}", deserialized_payload);
                        match deserialized_payload.payload.unwrap() {
                            AppPayload::NewCount(new_count) => count_state_handle.set(new_count),
                        }
                    })
                        as Box<dyn FnMut(JsValue)>);
                    let _unlisten = listen(
                        AppEventName::CounterChanged.into(),
                        handle_listen.as_ref().unchecked_ref(),
                    );
                    handle_listen.forget();
                }
                || {}
            }
        },
        (),
    );

    let handle_emit_event = |_| emit("event-name");

    let handle_increment_count = |_| {
        wasm_bindgen_futures::spawn_local(async {
            let new_count_js_value = JsFuture::from(invoke("increment_count")).await.unwrap();
            let new_count = JsValue::as_f64(&new_count_js_value).unwrap();
            info!(
                "new count received from command invocation itself = {:?}",
                new_count
            );
        })
    };

    html! {
        <div>
            <button onclick={handle_emit_event}>
                {"Emit event"}
            </button>
            <button onclick={handle_increment_count}>
                {"Increment count"}
            </button>
            <p>
                {"Current count: "}
                {*count_state_handle}
            </p>
        </div>
    }
}
