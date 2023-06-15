use leptos::{component, view, IntoView, Scope, spawn_local, create_signal, SignalSet};

use crate::audio::set_up_audio_stream;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (_, set_audio) = create_signal(cx, None);

    view! { cx,
        <p>"Sine wave: "</p>
        <button
            on:click=move |_| {
                spawn_local(async move {
                    let audio_out = set_up_audio_stream().await;
                    set_audio.set(Some(audio_out));
                });
            }
        >
            "Start"
        </button>
        <button
            on:click=move |_| {
              set_audio.set(None);
            }
        >
            "Stop"
        </button>
    }
}
