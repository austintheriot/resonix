use leptos::*;

mod app;
mod audio;

use app::App;

fn main() {
    mount_to_body(|cx| view! { cx, <App /> });
}
