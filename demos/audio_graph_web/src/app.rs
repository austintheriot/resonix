use leptos::{
    component, create_signal, spawn_local, view, IntoView, Scope, SignalSet, SignalUpdate,
};

use crate::audio::set_up_audio_graph;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (_, set_audio_context) = create_signal(cx, None);

    view! { cx,
        <p>"Sine wave: "</p>
        <button
            on:click=move |_| {
                spawn_local(async move {
                    let audio_context = set_up_audio_graph().await.unwrap();
                    set_audio_context.set(Some(audio_context));
                });
            }
        >
            "Start"
        </button>
        <button
            on:click=move |_| {
              set_audio_context.update(|audio_context| {
                if let Some(audio_context) = audio_context {
                    audio_context.uninitialize_dac();
                }
              });
            }
        >
            "Stop"
        </button>
    }
}
