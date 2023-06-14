use leptos::{component, view, IntoView, Scope, spawn_local};

use crate::audio::set_up_audio_stream;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    spawn_local(async {
        let audio_out = set_up_audio_stream().await;
        let audio_out = Box::new(audio_out);
        Box::leak(audio_out);
    });
    
    view! { cx,
        <p>"This is a sine wave!"</p>
    }
}
