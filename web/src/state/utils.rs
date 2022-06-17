use js_sys::Date;
use serde::Serialize;
use super::{app_action::AppAction, app_state::AppState};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    /// Used for logging state updates as JavaScript objects (to prevent unnecessarily long, stringified logs)
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log_with_styling(label: &str, styling: &str, object: &JsValue);
}

#[derive(Serialize)]
struct StateUpdateLog {
    time: String,
    action: AppAction,
    prev_state: AppState,
    next_state: AppState,
}

//// Logs basic information about action that caused state update as well as previous and next states
pub fn log_state_update(action: AppAction, prev_state: AppState, next_state: AppState) {
    let state_update_log = StateUpdateLog {
        time: Date::new(&JsValue::from_f64(Date::now()))
            .to_string()
            .into(),
        action,
        prev_state,
        next_state,
    };
    js_log_with_styling(
        "%cSTATE UPDATE:",
        "background-color: blue; color: white; padding: 0 5px;",
        &JsValue::from_serde(&state_update_log).unwrap(),
    );
}