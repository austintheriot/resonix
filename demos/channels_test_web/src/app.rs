use leptos::{component, create_signal, spawn_local, view, IntoView, Scope, SignalSet};

use crate::audio::set_up_audio_context;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (_, set_audio_context) = create_signal(cx, None);

    view! { cx,
        <p>"Sine wave: "</p>
        <button
            on:click=move |_| {
                spawn_local(async move {
                    let audio_context = set_up_audio_context().await;
                    set_audio_context.set(Some(audio_context));
                });
            }
        >
            "Start"
        </button>
        <button
            on:click=move |_| {
              set_audio_context.set(None);
            }
        >
            "Stop"
        </button>
    }
}
