#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use native_common::{AppEventName, AppPayload};
use std::{sync::Mutex};
use tauri::Manager;

#[derive(Default)]
struct AppState {
    count: Mutex<u32>,
}

#[tauri::command]
fn increment_count(state: tauri::State<AppState>, app_handle: tauri::AppHandle) -> u32 {
    println!("Received increment_count command!");
    let mut count_guard = state.count.lock().unwrap();
    *count_guard += 1;

    // emit a global event in response to this event
    app_handle
        .emit_all(
            AppEventName::CounterChanged.into(),
            AppPayload::NewCount(*count_guard),
        )
        .unwrap();

    let new_count = *count_guard;
    println!("new_count = {}", new_count);
    new_count
}

#[tauri::command]
fn get_count(state: tauri::State<AppState>) -> u32 {
    *state.count.lock().unwrap()
}

fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![increment_count, get_count])
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .setup(|app| {
            // listen to the `event-name` (emitted on any window)
            app.listen_global("event-name", move |event| {
                println!("got event-name with payload {:?}", event.payload());
            });

            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
