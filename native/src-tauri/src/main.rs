#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::Manager;

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct ExamplePayload {
    message: String,
}

#[derive(Default)]
struct AppState {
    count: Mutex<u32>,
}

#[tauri::command]
fn increment(state: tauri::State<AppState>) -> u32 {
    println!("Received increment command!");
    let mut count_gaurd = state.count.lock().unwrap();
    *count_gaurd += 1;

    let new_count= *count_gaurd;
    println!("new_count = {}", new_count);
    new_count
}

fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![increment])
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .setup(|app| {
            // listen to the `event-name` (emitted on any window)
            let id = app.listen_global("event-name", |event| {
                println!("got event-name with payload {:?}", event.payload());
            });

            // unlisten to the event using the `id` returned on the `listen_global` function
            // an `once_global` API is also exposed on the `App` struct
            // app.unlisten(id);

            // emit the `event-name` event to all webview windows on the frontend
            app.emit_all(
                "event-name",
                ExamplePayload {
                    message: "Tauri is awesome!".into(),
                },
            )
            .unwrap();
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
